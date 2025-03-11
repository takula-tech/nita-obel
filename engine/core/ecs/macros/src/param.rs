use crate::obel_ecs_path;
use obel_reflect_utils::ensure_no_collision;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{format, vec::Vec};
use syn::{
    ConstParam, Data, DataStruct, DeriveInput, GenericParam, Index, TypeParam, parse_quote, parse2,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

pub fn derive_system_param_impl(input: TokenStream) -> TokenStream {
    let path = obel_ecs_path();
    let token_stream = input.clone();
    let ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let Data::Struct(DataStruct {
        fields: field_definitions,
        ..
    }) = ast.data
    else {
        return syn::Error::new(ast.span(), "Invalid `SystemParam` type: expected a `struct`")
            .into_compile_error();
    };

    let mut field_locals = Vec::new();
    let mut fields = Vec::new();
    let mut field_types = Vec::new();
    for (i, field) in field_definitions.iter().enumerate() {
        field_locals.push(format_ident!("f{i}"));
        let i = Index::from(i);
        fields.push(field.ident.as_ref().map(|f| quote! { #f }).unwrap_or_else(|| quote! { #i }));
        field_types.push(&field.ty);
    }

    let generics = ast.generics;

    // Emit an error if there's any unrecognized lifetime names.
    for lt in generics.lifetimes() {
        let ident = &lt.lifetime.ident;
        let w = format_ident!("w");
        let s = format_ident!("s");
        if ident != &w && ident != &s {
            return syn::Error::new_spanned(
                lt,
                r#"invalid lifetime name: expected `'w` or `'s`
'w -- refers to data stored in the World.
's -- refers to data stored in the SystemParam's state.'"#,
            )
            .into_compile_error();
        }
    }

    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let lifetimeless_generics: Vec<_> =
        generics.params.iter().filter(|g| !matches!(g, GenericParam::Lifetime(_))).collect();

    let shadowed_lifetimes: Vec<_> = generics.lifetimes().map(|_| quote!('_)).collect();

    let mut punctuated_generics = Punctuated::<_, Comma>::new();
    punctuated_generics.extend(lifetimeless_generics.iter().map(|g| match g {
        GenericParam::Type(g) => GenericParam::Type(TypeParam {
            default: None,
            ..g.clone()
        }),
        GenericParam::Const(g) => GenericParam::Const(ConstParam {
            default: None,
            ..g.clone()
        }),
        _ => unreachable!(),
    }));

    let mut punctuated_generic_idents = Punctuated::<_, Comma>::new();
    punctuated_generic_idents.extend(lifetimeless_generics.iter().map(|g| match g {
        GenericParam::Type(g) => &g.ident,
        GenericParam::Const(g) => &g.ident,
        _ => unreachable!(),
    }));

    let punctuated_generics_no_bounds: Punctuated<_, Comma> = lifetimeless_generics
        .iter()
        .map(|&g| match g.clone() {
            GenericParam::Type(mut g) => {
                g.bounds.clear();
                GenericParam::Type(g)
            }
            g => g,
        })
        .collect();

    let mut tuple_types: Vec<_> = field_types.iter().map(|x| quote! { #x }).collect();
    let mut tuple_patterns: Vec<_> = field_locals.iter().map(|x| quote! { #x }).collect();

    // If the number of fields exceeds the 16-parameter limit,
    // fold the fields into tuples of tuples until we are below the limit.
    const LIMIT: usize = 16;
    while tuple_types.len() > LIMIT {
        let end = Vec::from_iter(tuple_types.drain(..LIMIT));
        tuple_types.push(parse_quote!( (#(#end,)*) ));

        let end = Vec::from_iter(tuple_patterns.drain(..LIMIT));
        tuple_patterns.push(parse_quote!( (#(#end,)*) ));
    }

    // Create a where clause for the `ReadOnlySystemParam` impl.
    // Ensure that each field implements `ReadOnlySystemParam`.
    let mut read_only_generics = generics.clone();
    let read_only_where_clause = read_only_generics.make_where_clause();
    for field_type in &field_types {
        read_only_where_clause
            .predicates
            .push(syn::parse_quote!(#field_type: #path::system::ReadOnlySystemParam));
    }

    let fields_alias =
        ensure_no_collision(format_ident!("__StructFieldsAlias"), token_stream.clone());

    let struct_name = &ast.ident;
    let state_struct_visibility = &ast.vis;
    let state_struct_name = ensure_no_collision(format_ident!("FetchState"), token_stream);

    let mut builder_name = None;
    for meta in ast.attrs.iter().filter(|a| a.path().is_ident("system_param")) {
        if let Err(e) = meta.parse_nested_meta(|nested| {
            if nested.path.is_ident("builder") {
                builder_name = Some(format_ident!("{struct_name}Builder"));
                Ok(())
            } else {
                Err(nested.error("Unsupported attribute"))
            }
        }) {
            return e.into_compile_error();
        }
    }

    let builder = builder_name.map(|builder_name| {
      let builder_type_parameters: Vec<_> = (0..fields.len()).map(|i| format_ident!("B{i}")).collect();
      let builder_doc_comment = format!("A [`SystemParamBuilder`] for a [`{struct_name}`].");
      let builder_struct = quote! {
          #[doc = #builder_doc_comment]
          struct #builder_name<#(#builder_type_parameters,)*> {
              #(#fields: #builder_type_parameters,)*
          }
      };
      let lifetimes: Vec<_> = generics.lifetimes().collect();
      let generic_struct = quote!{ #struct_name <#(#lifetimes,)* #punctuated_generic_idents> };
      let builder_impl = quote!{
          // SAFETY: This delegates to the `SystemParamBuilder` for tuples.
          unsafe impl<
              #(#lifetimes,)*
              #(#builder_type_parameters: #path::system::SystemParamBuilder<#field_types>,)*
              #punctuated_generics
          > #path::system::SystemParamBuilder<#generic_struct> for #builder_name<#(#builder_type_parameters,)*>
              #where_clause
          {
              fn build(self, world: &mut #path::world::World, meta: &mut #path::system::SystemMeta) -> <#generic_struct as #path::system::SystemParam>::State {
                  let #builder_name { #(#fields: #field_locals,)* } = self;
                  #state_struct_name {
                      state: #path::system::SystemParamBuilder::build((#(#tuple_patterns,)*), world, meta)
                  }
              }
          }
      };
      (builder_struct, builder_impl)
  });
    let (builder_struct, builder_impl) = builder.unzip();

    quote! {
        // We define the FetchState struct in an anonymous scope to avoid polluting the user namespace.
        // The struct can still be accessed via SystemParam::State, e.g. EventReaderState can be accessed via
        // <EventReader<'static, 'static, T> as SystemParam>::State
        const _: () = {
            // Allows rebinding the lifetimes of each field type.
            type #fields_alias <'w, 's, #punctuated_generics_no_bounds> = (#(#tuple_types,)*);

            #[doc(hidden)]
            #state_struct_visibility struct #state_struct_name <#(#lifetimeless_generics,)*>
            #where_clause {
                state: <#fields_alias::<'static, 'static, #punctuated_generic_idents> as #path::system::SystemParam>::State,
            }

            unsafe impl<#punctuated_generics> #path::system::SystemParam for
                #struct_name <#(#shadowed_lifetimes,)* #punctuated_generic_idents> #where_clause
            {
                type State = #state_struct_name<#punctuated_generic_idents>;
                type Item<'w, 's> = #struct_name #ty_generics;

                fn init_state(world: &mut #path::world::World, system_meta: &mut #path::system::SystemMeta) -> Self::State {
                    #state_struct_name {
                        state: <#fields_alias::<'_, '_, #punctuated_generic_idents> as #path::system::SystemParam>::init_state(world, system_meta),
                    }
                }

                unsafe fn new_archetype(state: &mut Self::State, archetype: &#path::archetype::Archetype, system_meta: &mut #path::system::SystemMeta) {
                    // SAFETY: The caller ensures that `archetype` is from the World the state was initialized from in `init_state`.
                    unsafe { <#fields_alias::<'_, '_, #punctuated_generic_idents> as #path::system::SystemParam>::new_archetype(&mut state.state, archetype, system_meta) }
                }

                fn apply(state: &mut Self::State, system_meta: &#path::system::SystemMeta, world: &mut #path::world::World) {
                    <#fields_alias::<'_, '_, #punctuated_generic_idents> as #path::system::SystemParam>::apply(&mut state.state, system_meta, world);
                }

                fn queue(state: &mut Self::State, system_meta: &#path::system::SystemMeta, world: #path::world::DeferredWorld) {
                    <#fields_alias::<'_, '_, #punctuated_generic_idents> as #path::system::SystemParam>::queue(&mut state.state, system_meta, world);
                }

                #[inline]
                unsafe fn validate_param<'w, 's>(
                    state: &'s Self::State,
                    system_meta: &#path::system::SystemMeta,
                    world: #path::world::unsafe_world_cell::UnsafeWorldCell<'w>,
                ) -> bool {
                    <(#(#tuple_types,)*) as #path::system::SystemParam>::validate_param(&state.state, system_meta, world)
                }

                #[inline]
                unsafe fn get_param<'w, 's>(
                    state: &'s mut Self::State,
                    system_meta: &#path::system::SystemMeta,
                    world: #path::world::unsafe_world_cell::UnsafeWorldCell<'w>,
                    change_tick: #path::component::Tick,
                ) -> Self::Item<'w, 's> {
                    let (#(#tuple_patterns,)*) = <
                        (#(#tuple_types,)*) as #path::system::SystemParam
                    >::get_param(&mut state.state, system_meta, world, change_tick);
                    #struct_name {
                        #(#fields: #field_locals,)*
                    }
                }
            }

            // Safety: Each field is `ReadOnlySystemParam`, so this can only read from the `World`
            unsafe impl<'w, 's, #punctuated_generics> #path::system::ReadOnlySystemParam for #struct_name #ty_generics #read_only_where_clause {}

            #builder_impl
        };

        #builder_struct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use quote::quote;

    #[track_caller]
    fn assert_formatted_eq(actual: TokenStream, expected: &str) {
        let syntax_tree: syn::File = parse2(actual).unwrap();
        let pretty = prettyplease::unparse(&syntax_tree);
        assert_eq!(pretty, expected, "\n === Pretty Please ===\n{}", pretty);
    }

    #[test]
    fn test_basic_struct() {
        let expected = indoc! {r#"
          const _: () = {
              type __StructFieldsAlias<'w, 's> = (Query<'w, 's, ()>, Local<'s, usize>);
              #[doc(hidden)]
              pub struct FetchState {
                  state: <__StructFieldsAlias<
                      'static,
                      'static,
                  > as obel_ecs::system::SystemParam>::State,
              }
              unsafe impl obel_ecs::system::SystemParam for CustomParam<'_, '_> {
                  type State = FetchState;
                  type Item<'w, 's> = CustomParam<'w, 's>;
                  fn init_state(
                      world: &mut obel_ecs::world::World,
                      system_meta: &mut obel_ecs::system::SystemMeta,
                  ) -> Self::State {
                      FetchState {
                          state: <__StructFieldsAlias<
                              '_,
                              '_,
                          > as obel_ecs::system::SystemParam>::init_state(world, system_meta),
                      }
                  }
                  unsafe fn new_archetype(
                      state: &mut Self::State,
                      archetype: &obel_ecs::archetype::Archetype,
                      system_meta: &mut obel_ecs::system::SystemMeta,
                  ) {
                      unsafe {
                          <__StructFieldsAlias<
                              '_,
                              '_,
                          > as obel_ecs::system::SystemParam>::new_archetype(
                              &mut state.state,
                              archetype,
                              system_meta,
                          )
                      }
                  }
                  fn apply(
                      state: &mut Self::State,
                      system_meta: &obel_ecs::system::SystemMeta,
                      world: &mut obel_ecs::world::World,
                  ) {
                      <__StructFieldsAlias<
                          '_,
                          '_,
                      > as obel_ecs::system::SystemParam>::apply(
                          &mut state.state,
                          system_meta,
                          world,
                      );
                  }
                  fn queue(
                      state: &mut Self::State,
                      system_meta: &obel_ecs::system::SystemMeta,
                      world: obel_ecs::world::DeferredWorld,
                  ) {
                      <__StructFieldsAlias<
                          '_,
                          '_,
                      > as obel_ecs::system::SystemParam>::queue(
                          &mut state.state,
                          system_meta,
                          world,
                      );
                  }
                  #[inline]
                  unsafe fn validate_param<'w, 's>(
                      state: &'s Self::State,
                      system_meta: &obel_ecs::system::SystemMeta,
                      world: obel_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
                  ) -> bool {
                      <(
                          Query<'w, 's, ()>,
                          Local<'s, usize>,
                      ) as obel_ecs::system::SystemParam>::validate_param(
                          &state.state,
                          system_meta,
                          world,
                      )
                  }
                  #[inline]
                  unsafe fn get_param<'w, 's>(
                      state: &'s mut Self::State,
                      system_meta: &obel_ecs::system::SystemMeta,
                      world: obel_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
                      change_tick: obel_ecs::component::Tick,
                  ) -> Self::Item<'w, 's> {
                      let (f0, f1) = <(
                          Query<'w, 's, ()>,
                          Local<'s, usize>,
                      ) as obel_ecs::system::SystemParam>::get_param(
                          &mut state.state,
                          system_meta,
                          world,
                          change_tick,
                      );
                      CustomParam {
                          query: f0,
                          local: f1,
                      }
                  }
              }
              unsafe impl<'w, 's> obel_ecs::system::ReadOnlySystemParam for CustomParam<'w, 's>
              where
                  Query<'w, 's, ()>: obel_ecs::system::ReadOnlySystemParam,
                  Local<'s, usize>: obel_ecs::system::ReadOnlySystemParam,
              {}
              unsafe impl<
                  'w,
                  's,
                  B0: obel_ecs::system::SystemParamBuilder<Query<'w, 's, ()>>,
                  B1: obel_ecs::system::SystemParamBuilder<Local<'s, usize>>,
              > obel_ecs::system::SystemParamBuilder<CustomParam<'w, 's>>
              for CustomParamBuilder<B0, B1> {
                  fn build(
                      self,
                      world: &mut obel_ecs::world::World,
                      meta: &mut obel_ecs::system::SystemMeta,
                  ) -> <CustomParam<'w, 's> as obel_ecs::system::SystemParam>::State {
                      let CustomParamBuilder { query: f0, local: f1 } = self;
                      FetchState {
                          state: obel_ecs::system::SystemParamBuilder::build((f0, f1), world, meta),
                      }
                  }
              }
          };
          ///A [`SystemParamBuilder`] for a [`CustomParam`].
          struct CustomParamBuilder<B0, B1> {
              query: B0,
              local: B1,
          }
        "#};

        let actual = derive_system_param_impl(quote! {
          #[derive(SystemParam)]
          #[system_param(builder)]
          pub struct CustomParam<'w, 's> {
              query: Query<'w, 's, ()>,
              local: Local<'s, usize>,
          }
        });

        assert_formatted_eq(actual, expected);
    }
}
