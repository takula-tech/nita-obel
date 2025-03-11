use std::{format, vec::Vec};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Field, Index, Meta, parse_quote, parse2,
    punctuated::Punctuated,
    token::{self, Comma},
};

use crate::{
    obel_ecs_path,
    query::{item_struct, world_query_impl},
};
use obel_reflect_utils::ensure_no_collision;

#[derive(Default)]
struct QueryDataAttributes {
    pub is_mutable: bool,

    pub derive_args: Punctuated<Meta, Comma>,
}

static MUTABLE_ATTRIBUTE_NAME: &str = "mutable";
static DERIVE_ATTRIBUTE_NAME: &str = "derive";

mod field_attr_keywords {
    syn::custom_keyword!(ignore);
}

pub static QUERY_DATA_ATTRIBUTE_NAME: &str = "query_data";

pub fn derive_query_data_impl(input: TokenStream) -> TokenStream {
    let path = obel_ecs_path();
    let tokens = input.clone();

    let ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let visibility = ast.vis;

    let mut attributes = QueryDataAttributes::default();
    for attr in &ast.attrs {
        if attr.path().get_ident().is_none_or(|ident| ident != QUERY_DATA_ATTRIBUTE_NAME) {
            continue;
        }

        let result = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(MUTABLE_ATTRIBUTE_NAME) {
                attributes.is_mutable = true;
                if meta.input.peek(token::Paren) {
                    Err(meta.error(format_args!("`{MUTABLE_ATTRIBUTE_NAME}` does not take any arguments")))
                } else {
                    Ok(())
                }
            } else if meta.path.is_ident(DERIVE_ATTRIBUTE_NAME) {
                meta.parse_nested_meta(|meta| {
                    attributes.derive_args.push(Meta::Path(meta.path));
                    Ok(())
                }).map_err(|_| {
                    meta.error(format_args!("`{DERIVE_ATTRIBUTE_NAME}` requires at least one argument"))
                })
            } else {
                Err(meta.error(format_args!("invalid attribute, expected `{MUTABLE_ATTRIBUTE_NAME}` or `{DERIVE_ATTRIBUTE_NAME}`")))
            }
        });

        if let Err(err) = result {
            return err.to_compile_error();
        }
    }

    let user_generics = ast.generics.clone();
    let (user_impl_generics, user_ty_generics, user_where_clauses) = user_generics.split_for_impl();
    let user_generics_with_world = {
        let mut generics = ast.generics;
        generics.params.insert(0, parse_quote!('__w));
        generics
    };
    let (user_impl_generics_with_world, user_ty_generics_with_world, user_where_clauses_with_world) =
        user_generics_with_world.split_for_impl();

    let struct_name = ast.ident;
    let read_only_struct_name = if attributes.is_mutable {
        Ident::new(&format!("{struct_name}ReadOnly"), Span::call_site())
    } else {
        struct_name.clone()
    };

    let item_struct_name = Ident::new(&format!("{struct_name}Item"), Span::call_site());
    let read_only_item_struct_name = if attributes.is_mutable {
        Ident::new(&format!("{struct_name}ReadOnlyItem"), Span::call_site())
    } else {
        item_struct_name.clone()
    };

    let fetch_struct_name = Ident::new(&format!("{struct_name}Fetch"), Span::call_site());
    let fetch_struct_name = ensure_no_collision(fetch_struct_name, tokens.clone());
    let read_only_fetch_struct_name = if attributes.is_mutable {
        let new_ident = Ident::new(&format!("{struct_name}ReadOnlyFetch"), Span::call_site());
        ensure_no_collision(new_ident, tokens.clone())
    } else {
        fetch_struct_name.clone()
    };

    let marker_name =
        ensure_no_collision(format_ident!("_world_query_derive_marker"), tokens.clone());

    // Generate a name for the state struct that doesn't conflict
    // with the struct definition.
    let state_struct_name = Ident::new(&format!("{struct_name}State"), Span::call_site());
    let state_struct_name = ensure_no_collision(state_struct_name, tokens);

    let Data::Struct(DataStruct {
        fields,
        ..
    }) = &ast.data
    else {
        return syn::Error::new(Span::call_site(), "#[derive(QueryData)]` only supports structs")
            .into_compile_error();
    };

    let mut field_attrs = Vec::new();
    let mut field_visibilities = Vec::new();
    let mut field_idents = Vec::new();
    let mut named_field_idents = Vec::new();
    let mut field_types = Vec::new();
    let mut read_only_field_types = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let attrs = match read_world_query_field_info(field) {
            Ok(QueryDataFieldInfo {
                attrs,
            }) => attrs,
            Err(e) => return e.into_compile_error(),
        };

        let named_field_ident =
            field.ident.as_ref().cloned().unwrap_or_else(|| format_ident!("f{i}"));
        let i = Index::from(i);
        let field_ident = field.ident.as_ref().map_or(quote! { #i }, |i| quote! { #i });
        field_idents.push(field_ident);
        named_field_idents.push(named_field_ident);
        field_attrs.push(attrs);
        field_visibilities.push(field.vis.clone());
        let field_ty = field.ty.clone();
        field_types.push(quote!(#field_ty));
        read_only_field_types.push(quote!(<#field_ty as #path::query::QueryData>::ReadOnly));
    }

    let derive_args = &attributes.derive_args;
    // `#[derive()]` is valid syntax
    let derive_macro_call = quote! { #[derive(#derive_args)] };

    let mutable_item_struct = item_struct(
        &path,
        fields,
        &derive_macro_call,
        &struct_name,
        &visibility,
        &item_struct_name,
        &field_types,
        &user_impl_generics_with_world,
        &field_attrs,
        &field_visibilities,
        &field_idents,
        &user_ty_generics,
        &user_ty_generics_with_world,
        user_where_clauses_with_world,
    );
    let mutable_world_query_impl = world_query_impl(
        &path,
        &struct_name,
        &visibility,
        &fetch_struct_name,
        &field_types,
        &user_impl_generics,
        &user_impl_generics_with_world,
        &user_ty_generics,
        &user_ty_generics_with_world,
        &named_field_idents,
        &marker_name,
        &state_struct_name,
        user_where_clauses,
        user_where_clauses_with_world,
    );

    let (read_only_struct, read_only_impl) = if attributes.is_mutable {
        // If the query is mutable, we need to generate a separate readonly version of some things
        let readonly_item_struct = item_struct(
            &path,
            fields,
            &derive_macro_call,
            &read_only_struct_name,
            &visibility,
            &read_only_item_struct_name,
            &read_only_field_types,
            &user_impl_generics_with_world,
            &field_attrs,
            &field_visibilities,
            &field_idents,
            &user_ty_generics,
            &user_ty_generics_with_world,
            user_where_clauses_with_world,
        );
        let readonly_world_query_impl = world_query_impl(
            &path,
            &read_only_struct_name,
            &visibility,
            &read_only_fetch_struct_name,
            &read_only_field_types,
            &user_impl_generics,
            &user_impl_generics_with_world,
            &user_ty_generics,
            &user_ty_generics_with_world,
            &named_field_idents,
            &marker_name,
            &state_struct_name,
            user_where_clauses,
            user_where_clauses_with_world,
        );
        let read_only_structs = quote! {
            #[doc = concat!(
                "Automatically generated [`WorldQuery`](",
                stringify!(#path),
                "::query::WorldQuery) type for a read-only variant of [`",
                stringify!(#struct_name),
                "`]."
            )]
            #[automatically_derived]
            #visibility struct #read_only_struct_name #user_impl_generics #user_where_clauses {
                #(
                    #[doc = "Automatically generated read-only field for accessing `"]
                    #[doc = stringify!(#field_types)]
                    #[doc = "`."]
                    #field_visibilities #named_field_idents: #read_only_field_types,
                )*
            }

            #readonly_item_struct
        };
        (read_only_structs, readonly_world_query_impl)
    } else {
        (quote! {}, quote! {})
    };

    let data_impl = {
        let read_only_data_impl = if attributes.is_mutable {
            quote! {
                /// SAFETY: we assert fields are readonly below
                unsafe impl #user_impl_generics #path::query::QueryData
                for #read_only_struct_name #user_ty_generics #user_where_clauses {
                    const IS_READ_ONLY: bool = true;
                    type ReadOnly = #read_only_struct_name #user_ty_generics;
                    type Item<'__w> = #read_only_item_struct_name #user_ty_generics_with_world;

                    fn shrink<'__wlong: '__wshort, '__wshort>(
                        item: Self::Item<'__wlong>
                    ) -> Self::Item<'__wshort> {
                        #read_only_item_struct_name {
                            #(
                                #field_idents: <#read_only_field_types>::shrink(item.#field_idents),
                            )*
                        }
                    }

                    /// SAFETY: we call `fetch` for each member that implements `Fetch`.
                    #[inline(always)]
                    unsafe fn fetch<'__w>(
                        _fetch: &mut <Self as #path::query::WorldQuery>::Fetch<'__w>,
                        _entity: #path::entity::Entity,
                        _table_row: #path::storage::TableRow,
                    ) -> Self::Item<'__w> {
                        Self::Item {
                            #(#field_idents: <#read_only_field_types>::fetch(&mut _fetch.#named_field_idents, _entity, _table_row),)*
                        }
                    }
                }
            }
        } else {
            quote! {}
        };

        let is_read_only = !attributes.is_mutable;

        quote! {
            /// SAFETY: we assert fields are readonly below
            unsafe impl #user_impl_generics #path::query::QueryData
            for #struct_name #user_ty_generics #user_where_clauses {
                const IS_READ_ONLY: bool = #is_read_only;
                type ReadOnly = #read_only_struct_name #user_ty_generics;
                type Item<'__w> = #item_struct_name #user_ty_generics_with_world;

                fn shrink<'__wlong: '__wshort, '__wshort>(
                    item: Self::Item<'__wlong>
                ) -> Self::Item<'__wshort> {
                    #item_struct_name {
                        #(
                            #field_idents: <#field_types>::shrink(item.#field_idents),
                        )*
                    }
                }

                /// SAFETY: we call `fetch` for each member that implements `Fetch`.
                #[inline(always)]
                unsafe fn fetch<'__w>(
                    _fetch: &mut <Self as #path::query::WorldQuery>::Fetch<'__w>,
                    _entity: #path::entity::Entity,
                    _table_row: #path::storage::TableRow,
                ) -> Self::Item<'__w> {
                    Self::Item {
                        #(#field_idents: <#field_types>::fetch(&mut _fetch.#named_field_idents, _entity, _table_row),)*
                    }
                }
            }

            #read_only_data_impl
        }
    };

    let read_only_data_impl = quote! {
        /// SAFETY: we assert fields are readonly below
        unsafe impl #user_impl_generics #path::query::ReadOnlyQueryData
        for #read_only_struct_name #user_ty_generics #user_where_clauses {}
    };

    let read_only_asserts = if attributes.is_mutable {
        quote! {
            // Double-check that the data fetched by `<_ as WorldQuery>::ReadOnly` is read-only.
            // This is technically unnecessary as `<_ as WorldQuery>::ReadOnly: ReadOnlyQueryData`
            // but to protect against future mistakes we assert the assoc type implements `ReadOnlyQueryData` anyway
            #( assert_readonly::<#read_only_field_types>(); )*
        }
    } else {
        quote! {
            // Statically checks that the safety guarantee of `ReadOnlyQueryData` for `$fetch_struct_name` actually holds true.
            // We need this to make sure that we don't compile `ReadOnlyQueryData` if our struct contains nested `QueryData`
            // members that don't implement it. I.e.:
            // ```
            // #[derive(QueryData)]
            // pub struct Foo { a: &'static mut MyComponent }
            // ```
            #( assert_readonly::<#field_types>(); )*
        }
    };

    let data_asserts = quote! {
        #( assert_data::<#field_types>(); )*
    };

    quote! {
        #mutable_item_struct

        #read_only_struct

        const _: () = {
            #[doc(hidden)]
            #[doc = concat!(
                "Automatically generated internal [`WorldQuery`](",
                stringify!(#path),
                "::query::WorldQuery) state type for [`",
                stringify!(#struct_name),
                "`], used for caching."
            )]
            #[automatically_derived]
            #visibility struct #state_struct_name #user_impl_generics #user_where_clauses {
                #(#named_field_idents: <#field_types as #path::query::WorldQuery>::State,)*
            }

            #mutable_world_query_impl

            #read_only_impl

            #data_impl

            #read_only_data_impl
        };

        #[allow(dead_code)]
        const _: () = {
            fn assert_readonly<T>()
            where
                T: #path::query::ReadOnlyQueryData,
            {
            }

            fn assert_data<T>()
            where
                T: #path::query::QueryData,
            {
            }

            // We generate a readonly assertion for every struct member.
            fn assert_all #user_impl_generics_with_world () #user_where_clauses_with_world {
                #read_only_asserts
                #data_asserts
            }
        };

        // The original struct will most likely be left unused. As we don't want our users having
        // to specify `#[allow(dead_code)]` for their custom queries, we are using this cursed
        // workaround.
        #[allow(dead_code)]
        const _: () = {
            fn dead_code_workaround #user_impl_generics (
                q: #struct_name #user_ty_generics,
                q2: #read_only_struct_name #user_ty_generics
            ) #user_where_clauses {
                #(q.#field_idents;)*
                #(q2.#field_idents;)*
            }
        };
    }
}

struct QueryDataFieldInfo {
    /// All field attributes except for `query_data` ones.
    attrs: Vec<Attribute>,
}

fn read_world_query_field_info(field: &Field) -> syn::Result<QueryDataFieldInfo> {
    let mut attrs = Vec::new();
    for attr in &field.attrs {
        if attr.path().get_ident().is_some_and(|ident| ident == QUERY_DATA_ATTRIBUTE_NAME) {
            return Err(syn::Error::new_spanned(
                attr,
                "#[derive(QueryData)] does not support field attributes.",
            ));
        }
        attrs.push(attr.clone());
    }

    Ok(QueryDataFieldInfo {
        attrs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[track_caller]
    fn assert_formatted_eq(actual: TokenStream, expected: &str) {
        let syntax_tree: syn::File = parse2(actual).unwrap();
        let pretty = prettyplease::unparse(&syntax_tree);
        assert_eq!(pretty, expected, "\n === Pretty Please ===\n{}", pretty);
    }

    #[test]
    fn test_derive_query_data_impl() {
        let expected = indoc! {r#"
          #[derive(Debug)]
          #[doc = concat!(
              "Automatically generated [`WorldQuery`](", stringify!(obel_ecs),
              "::query::WorldQuery) item type for [`", stringify!(MyStruct),
              "`], returned when iterating over query results."
          )]
          #[automatically_derived]
          struct MyStructItem<'__w> {
              component_a: <&'static mut ComponentA as obel_ecs::query::QueryData>::Item<'__w>,
          }
          #[doc = concat!(
              "Automatically generated [`WorldQuery`](", stringify!(obel_ecs),
              "::query::WorldQuery) type for a read-only variant of [`", stringify!(MyStruct),
              "`]."
          )]
          #[automatically_derived]
          struct MyStructReadOnly {
              ///Automatically generated read-only field for accessing `
              #[doc = stringify!(& 'static mut ComponentA)]
              ///`.
              component_a: <&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly,
          }
          #[derive(Debug)]
          #[doc = concat!(
              "Automatically generated [`WorldQuery`](", stringify!(obel_ecs),
              "::query::WorldQuery) item type for [`", stringify!(MyStructReadOnly),
              "`], returned when iterating over query results."
          )]
          #[automatically_derived]
          struct MyStructReadOnlyItem<'__w> {
              component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly as obel_ecs::query::QueryData>::Item<
                  '__w,
              >,
          }
          const _: () = {
              #[doc(hidden)]
              #[doc = concat!(
                  "Automatically generated internal [`WorldQuery`](", stringify!(obel_ecs),
                  "::query::WorldQuery) state type for [`", stringify!(MyStruct),
                  "`], used for caching."
              )]
              #[automatically_derived]
              struct MyStructState {
                  component_a: <&'static mut ComponentA as obel_ecs::query::WorldQuery>::State,
              }
              #[doc(hidden)]
              #[doc = concat!(
                  "Automatically generated internal [`WorldQuery`](", stringify!(obel_ecs),
                  "::query::WorldQuery) fetch type for [`", stringify!(MyStruct),
                  "`], used to define the world data accessed by this query."
              )]
              #[automatically_derived]
              struct MyStructFetch<'__w> {
                  component_a: <&'static mut ComponentA as obel_ecs::query::WorldQuery>::Fetch<
                      '__w,
                  >,
                  _world_query_derive_marker: &'__w (),
              }
              impl<'__w> Clone for MyStructFetch<'__w> {
                  fn clone(&self) -> Self {
                      Self {
                          component_a: self.component_a.clone(),
                          _world_query_derive_marker: &(),
                      }
                  }
              }
              unsafe impl obel_ecs::query::WorldQuery for MyStruct {
                  type Fetch<'__w> = MyStructFetch<'__w>;
                  type State = MyStructState;
                  fn shrink_fetch<'__wlong: '__wshort, '__wshort>(
                      fetch: <MyStruct as obel_ecs::query::WorldQuery>::Fetch<'__wlong>,
                  ) -> <MyStruct as obel_ecs::query::WorldQuery>::Fetch<'__wshort> {
                      MyStructFetch {
                          component_a: <&'static mut ComponentA>::shrink_fetch(fetch.component_a),
                          _world_query_derive_marker: &(),
                      }
                  }
                  unsafe fn init_fetch<'__w>(
                      _world: obel_ecs::world::unsafe_world_cell::UnsafeWorldCell<'__w>,
                      state: &Self::State,
                      _last_run: obel_ecs::component::Tick,
                      _this_run: obel_ecs::component::Tick,
                  ) -> <Self as obel_ecs::query::WorldQuery>::Fetch<'__w> {
                      MyStructFetch {
                          component_a: <&'static mut ComponentA>::init_fetch(
                              _world,
                              &state.component_a,
                              _last_run,
                              _this_run,
                          ),
                          _world_query_derive_marker: &(),
                      }
                  }
                  const IS_DENSE: bool = true && <&'static mut ComponentA>::IS_DENSE;
                  /// SAFETY: we call `set_archetype` for each member that implements `Fetch`
                  #[inline]
                  unsafe fn set_archetype<'__w>(
                      _fetch: &mut <Self as obel_ecs::query::WorldQuery>::Fetch<'__w>,
                      _state: &Self::State,
                      _archetype: &'__w obel_ecs::archetype::Archetype,
                      _table: &'__w obel_ecs::storage::Table,
                  ) {
                      <&'static mut ComponentA>::set_archetype(
                          &mut _fetch.component_a,
                          &_state.component_a,
                          _archetype,
                          _table,
                      );
                  }
                  /// SAFETY: we call `set_table` for each member that implements `Fetch`
                  #[inline]
                  unsafe fn set_table<'__w>(
                      _fetch: &mut <Self as obel_ecs::query::WorldQuery>::Fetch<'__w>,
                      _state: &Self::State,
                      _table: &'__w obel_ecs::storage::Table,
                  ) {
                      <&'static mut ComponentA>::set_table(
                          &mut _fetch.component_a,
                          &_state.component_a,
                          _table,
                      );
                  }
                  fn update_component_access(
                      state: &Self::State,
                      _access: &mut obel_ecs::query::FilteredAccess<
                          obel_ecs::component::ComponentId,
                      >,
                  ) {
                      <&'static mut ComponentA>::update_component_access(
                          &state.component_a,
                          _access,
                      );
                  }
                  fn init_state(world: &mut obel_ecs::world::World) -> MyStructState {
                      MyStructState {
                          component_a: <&'static mut ComponentA>::init_state(world),
                      }
                  }
                  fn get_state(
                      components: &obel_ecs::component::Components,
                  ) -> Option<MyStructState> {
                      Some(MyStructState {
                          component_a: <&'static mut ComponentA>::get_state(components)?,
                      })
                  }
                  fn matches_component_set(
                      state: &Self::State,
                      _set_contains_id: &impl Fn(obel_ecs::component::ComponentId) -> bool,
                  ) -> bool {
                      true
                          && <&'static mut ComponentA>::matches_component_set(
                              &state.component_a,
                              _set_contains_id,
                          )
                  }
              }
              #[doc(hidden)]
              #[doc = concat!(
                  "Automatically generated internal [`WorldQuery`](", stringify!(obel_ecs),
                  "::query::WorldQuery) fetch type for [`", stringify!(MyStructReadOnly),
                  "`], used to define the world data accessed by this query."
              )]
              #[automatically_derived]
              struct MyStructReadOnlyFetch<'__w> {
                  component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly as obel_ecs::query::WorldQuery>::Fetch<
                      '__w,
                  >,
                  _world_query_derive_marker: &'__w (),
              }
              impl<'__w> Clone for MyStructReadOnlyFetch<'__w> {
                  fn clone(&self) -> Self {
                      Self {
                          component_a: self.component_a.clone(),
                          _world_query_derive_marker: &(),
                      }
                  }
              }
              unsafe impl obel_ecs::query::WorldQuery for MyStructReadOnly {
                  type Fetch<'__w> = MyStructReadOnlyFetch<'__w>;
                  type State = MyStructState;
                  fn shrink_fetch<'__wlong: '__wshort, '__wshort>(
                      fetch: <MyStructReadOnly as obel_ecs::query::WorldQuery>::Fetch<'__wlong>,
                  ) -> <MyStructReadOnly as obel_ecs::query::WorldQuery>::Fetch<'__wshort> {
                      MyStructReadOnlyFetch {
                          component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::shrink_fetch(
                              fetch.component_a,
                          ),
                          _world_query_derive_marker: &(),
                      }
                  }
                  unsafe fn init_fetch<'__w>(
                      _world: obel_ecs::world::unsafe_world_cell::UnsafeWorldCell<'__w>,
                      state: &Self::State,
                      _last_run: obel_ecs::component::Tick,
                      _this_run: obel_ecs::component::Tick,
                  ) -> <Self as obel_ecs::query::WorldQuery>::Fetch<'__w> {
                      MyStructReadOnlyFetch {
                          component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::init_fetch(
                              _world,
                              &state.component_a,
                              _last_run,
                              _this_run,
                          ),
                          _world_query_derive_marker: &(),
                      }
                  }
                  const IS_DENSE: bool = true
                      && <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::IS_DENSE;
                  /// SAFETY: we call `set_archetype` for each member that implements `Fetch`
                  #[inline]
                  unsafe fn set_archetype<'__w>(
                      _fetch: &mut <Self as obel_ecs::query::WorldQuery>::Fetch<'__w>,
                      _state: &Self::State,
                      _archetype: &'__w obel_ecs::archetype::Archetype,
                      _table: &'__w obel_ecs::storage::Table,
                  ) {
                      <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::set_archetype(
                          &mut _fetch.component_a,
                          &_state.component_a,
                          _archetype,
                          _table,
                      );
                  }
                  /// SAFETY: we call `set_table` for each member that implements `Fetch`
                  #[inline]
                  unsafe fn set_table<'__w>(
                      _fetch: &mut <Self as obel_ecs::query::WorldQuery>::Fetch<'__w>,
                      _state: &Self::State,
                      _table: &'__w obel_ecs::storage::Table,
                  ) {
                      <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::set_table(
                          &mut _fetch.component_a,
                          &_state.component_a,
                          _table,
                      );
                  }
                  fn update_component_access(
                      state: &Self::State,
                      _access: &mut obel_ecs::query::FilteredAccess<
                          obel_ecs::component::ComponentId,
                      >,
                  ) {
                      <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::update_component_access(
                          &state.component_a,
                          _access,
                      );
                  }
                  fn init_state(world: &mut obel_ecs::world::World) -> MyStructState {
                      MyStructState {
                          component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::init_state(
                              world,
                          ),
                      }
                  }
                  fn get_state(
                      components: &obel_ecs::component::Components,
                  ) -> Option<MyStructState> {
                      Some(MyStructState {
                          component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::get_state(
                              components,
                          )?,
                      })
                  }
                  fn matches_component_set(
                      state: &Self::State,
                      _set_contains_id: &impl Fn(obel_ecs::component::ComponentId) -> bool,
                  ) -> bool {
                      true
                          && <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::matches_component_set(
                              &state.component_a,
                              _set_contains_id,
                          )
                  }
              }
              /// SAFETY: we assert fields are readonly below
              unsafe impl obel_ecs::query::QueryData for MyStruct {
                  const IS_READ_ONLY: bool = false;
                  type ReadOnly = MyStructReadOnly;
                  type Item<'__w> = MyStructItem<'__w>;
                  fn shrink<'__wlong: '__wshort, '__wshort>(
                      item: Self::Item<'__wlong>,
                  ) -> Self::Item<'__wshort> {
                      MyStructItem {
                          component_a: <&'static mut ComponentA>::shrink(item.component_a),
                      }
                  }
                  /// SAFETY: we call `fetch` for each member that implements `Fetch`.
                  #[inline(always)]
                  unsafe fn fetch<'__w>(
                      _fetch: &mut <Self as obel_ecs::query::WorldQuery>::Fetch<'__w>,
                      _entity: obel_ecs::entity::Entity,
                      _table_row: obel_ecs::storage::TableRow,
                  ) -> Self::Item<'__w> {
                      Self::Item {
                          component_a: <&'static mut ComponentA>::fetch(
                              &mut _fetch.component_a,
                              _entity,
                              _table_row,
                          ),
                      }
                  }
              }
              /// SAFETY: we assert fields are readonly below
              unsafe impl obel_ecs::query::QueryData for MyStructReadOnly {
                  const IS_READ_ONLY: bool = true;
                  type ReadOnly = MyStructReadOnly;
                  type Item<'__w> = MyStructReadOnlyItem<'__w>;
                  fn shrink<'__wlong: '__wshort, '__wshort>(
                      item: Self::Item<'__wlong>,
                  ) -> Self::Item<'__wshort> {
                      MyStructReadOnlyItem {
                          component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::shrink(
                              item.component_a,
                          ),
                      }
                  }
                  /// SAFETY: we call `fetch` for each member that implements `Fetch`.
                  #[inline(always)]
                  unsafe fn fetch<'__w>(
                      _fetch: &mut <Self as obel_ecs::query::WorldQuery>::Fetch<'__w>,
                      _entity: obel_ecs::entity::Entity,
                      _table_row: obel_ecs::storage::TableRow,
                  ) -> Self::Item<'__w> {
                      Self::Item {
                          component_a: <<&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly>::fetch(
                              &mut _fetch.component_a,
                              _entity,
                              _table_row,
                          ),
                      }
                  }
              }
              /// SAFETY: we assert fields are readonly below
              unsafe impl obel_ecs::query::ReadOnlyQueryData for MyStructReadOnly {}
          };
          #[allow(dead_code)]
          const _: () = {
              fn assert_readonly<T>()
              where
                  T: obel_ecs::query::ReadOnlyQueryData,
              {}
              fn assert_data<T>()
              where
                  T: obel_ecs::query::QueryData,
              {}
              fn assert_all<'__w>() {
                  assert_readonly::<
                      <&'static mut ComponentA as obel_ecs::query::QueryData>::ReadOnly,
                  >();
                  assert_data::<&'static mut ComponentA>();
              }
          };
          #[allow(dead_code)]
          const _: () = {
              fn dead_code_workaround(q: MyStruct, q2: MyStructReadOnly) {
                  q.component_a;
                  q2.component_a;
              }
          };
      "#};

        let actual = derive_query_data_impl(quote! {
            #[query_data(mutable, derive(Debug))]
            struct MyStruct {
                component_a: &'static mut ComponentA,
            }
        });

        assert_formatted_eq(actual, expected);
    }
}
