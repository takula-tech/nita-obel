use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use std::{format, vec::Vec};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Ident, Member, Path, parse_quote, parse2,
    spanned::Spanned,
};

use relationship::*;
mod relationship;

use attr::*;
mod attr;

use require::*;
mod require;

pub fn derive_component_impl(input: TokenStream) -> TokenStream {
    let obel_ecs_path: Path = crate::obel_ecs_path();
    let mut ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let attrs = match parse_component_attr(&ast) {
        Ok(attrs) => attrs,
        Err(e) => return e.into_compile_error(),
    };

    let relationship = match derive_relationship(&ast, &attrs, &obel_ecs_path) {
        Ok(value) => value,
        Err(err) => err.into_compile_error().into(),
    };

    let relationship_target = match derive_relationship_target(&ast, &attrs, &obel_ecs_path) {
        Ok(value) => value,
        Err(err) => err.into_compile_error().into(),
    };

    let map_entities = map_entities(
      &ast.data,
      Ident::new("this", Span::call_site()),
      relationship.is_some(),
      relationship_target.is_some(),
  ).map(|map_entities_impl| quote! {
      fn map_entities<M: #obel_ecs_path::entity::EntityMapper>(this: &mut Self, mapper: &mut M) {
          use #obel_ecs_path::entity::MapEntities;
          #map_entities_impl
      }
});

    let storage = storage_path(&obel_ecs_path, attrs.storage);

    let on_add_path = attrs.on_add.map(|path| path.to_token_stream(&obel_ecs_path));
    let on_remove_path = attrs.on_remove.map(|path| path.to_token_stream(&obel_ecs_path));

    let on_insert_path = if relationship.is_some() {
        if attrs.on_insert.is_some() {
            return syn::Error::new(
                ast.span(),
                "Custom on_insert hooks are not supported as relationships already define an on_insert hook",
            )
            .into_compile_error();
        }

        Some(quote!(<Self as #obel_ecs_path::relationship::Relationship>::on_insert))
    } else {
        attrs.on_insert.map(|path| path.to_token_stream(&obel_ecs_path))
    };

    let on_replace_path = if relationship.is_some() {
        if attrs.on_replace.is_some() {
            return syn::Error::new(
                ast.span(),
                "Custom on_replace hooks are not supported as Relationships already define an on_replace hook",
            )
            .into_compile_error();
        }

        Some(quote!(<Self as #obel_ecs_path::relationship::Relationship>::on_replace))
    } else if attrs.relationship_target.is_some() {
        if attrs.on_replace.is_some() {
            return syn::Error::new(
                ast.span(),
                "Custom on_replace hooks are not supported as RelationshipTarget already defines an on_replace hook",
            )
            .into_compile_error();
        }

        Some(quote!(<Self as #obel_ecs_path::relationship::RelationshipTarget>::on_replace))
    } else {
        attrs.on_replace.map(|path| path.to_token_stream(&obel_ecs_path))
    };

    let on_despawn_path = if attrs.relationship_target.is_some_and(|target| target.linked_spawn) {
        if attrs.on_despawn.is_some() {
            return syn::Error::new(
                ast.span(),
                "Custom on_despawn hooks are not supported as this RelationshipTarget already defines an on_despawn hook, via the 'linked_spawn' attribute",
            )
            .into_compile_error();
        }

        Some(quote!(<Self as #obel_ecs_path::relationship::RelationshipTarget>::on_despawn))
    } else {
        attrs.on_despawn.map(|path| path.to_token_stream(&obel_ecs_path))
    };

    let on_add = hook_register_function_call(&obel_ecs_path, quote! {on_add}, on_add_path);
    let on_insert = hook_register_function_call(&obel_ecs_path, quote! {on_insert}, on_insert_path);
    let on_replace =
        hook_register_function_call(&obel_ecs_path, quote! {on_replace}, on_replace_path);
    let on_remove = hook_register_function_call(&obel_ecs_path, quote! {on_remove}, on_remove_path);
    let on_despawn =
        hook_register_function_call(&obel_ecs_path, quote! {on_despawn}, on_despawn_path);

    ast.generics.make_where_clause().predicates.push(parse_quote! { Self: Send + Sync + 'static });

    let requires = &attrs.requires;
    let mut register_required = Vec::with_capacity(attrs.requires.iter().len());
    let mut register_recursive_requires = Vec::with_capacity(attrs.requires.iter().len());
    if let Some(requires) = requires {
        for require in requires {
            let ident = &require.path;
            register_recursive_requires.push(quote! {
                <#ident as #obel_ecs_path::component::Component>::register_required_components(
                    requiree,
                    components,
                    required_components,
                    inheritance_depth + 1,
                    recursion_check_stack
                );
            });
            match &require.func {
                Some(func) => {
                    register_required.push(quote! {
                        components.register_required_components_manual::<Self, #ident>(
                            required_components,
                            || { let x: #ident = (#func)().into(); x },
                            inheritance_depth,
                            recursion_check_stack
                        );
                    });
                }
                None => {
                    register_required.push(quote! {
                        components.register_required_components_manual::<Self, #ident>(
                            required_components,
                            <#ident as Default>::default,
                            inheritance_depth,
                            recursion_check_stack
                        );
                    });
                }
            }
        }
    }
    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    let required_component_docs = attrs.requires.map(|r| {
        let paths = r
            .iter()
            .map(|r| format!("[`{}`]", r.path.to_token_stream()))
            .collect::<Vec<_>>()
            .join(", ");
        let doc = format!("**Required Components**: {paths}. \n\n A component's Required Components are inserted whenever it is inserted. Note that this will also insert the required components _of_ the required components, recursively, in depth-first order.");
        quote! {
            #[doc = #doc]
        }
    });

    let mutable_type = (attrs.immutable || relationship.is_some())
        .then_some(quote! { #obel_ecs_path::component::Immutable })
        .unwrap_or(quote! { #obel_ecs_path::component::Mutable });

    let clone_behavior = if relationship_target.is_some() {
        quote!(#obel_ecs_path::component::ComponentCloneBehavior::Custom(#obel_ecs_path::relationship::clone_relationship_target::<Self>))
    } else {
        quote!(
            use #obel_ecs_path::component::{DefaultCloneBehaviorBase, DefaultCloneBehaviorViaClone};
            (&&&#obel_ecs_path::component::DefaultCloneBehaviorSpecialization::<Self>::default()).default_clone_behavior()
        )
    };

    // This puts `register_required` before `register_recursive_requires` to ensure that the constructors of _all_ top
    // level components are initialized first, giving them precedence over recursively defined constructors for the same component type
    quote! {
        #required_component_docs
        impl #impl_generics #obel_ecs_path::component::Component for #struct_name #type_generics #where_clause {
            const STORAGE_TYPE: #obel_ecs_path::component::StorageType = #storage;
            type Mutability = #mutable_type;
            fn register_required_components(
                requiree: #obel_ecs_path::component::ComponentId,
                components: &mut #obel_ecs_path::component::ComponentsRegistrator,
                required_components: &mut #obel_ecs_path::component::RequiredComponents,
                inheritance_depth: u16,
                recursion_check_stack: &mut #obel_ecs_path::__macro_exports::Vec<#obel_ecs_path::component::ComponentId>
            ) {
                #obel_ecs_path::component::enforce_no_required_components_recursion(components, recursion_check_stack);
                let self_id = components.register_component::<Self>();
                recursion_check_stack.push(self_id);
                #(#register_required)*
                #(#register_recursive_requires)*
                recursion_check_stack.pop();
            }

            #on_add
            #on_insert
            #on_replace
            #on_remove
            #on_despawn

            fn clone_behavior() -> #obel_ecs_path::component::ComponentCloneBehavior {
                #clone_behavior
            }

            #map_entities
        }

        #relationship

        #relationship_target
    }
}

const ENTITIES: &str = "entities";

pub(crate) fn map_entities(
    data: &Data,
    self_ident: Ident,
    is_relationship: bool,
    is_relationship_target: bool,
) -> Option<TokenStream> {
    match data {
        Data::Struct(DataStruct {
            fields,
            ..
        }) => {
            let mut map = Vec::with_capacity(fields.len());

            let relationship = if is_relationship || is_relationship_target {
                relationship_field(fields, "MapEntities", fields.span()).ok()
            } else {
                None
            };
            fields
                .iter()
                .enumerate()
                .filter(|(_, field)| {
                    field.attrs.iter().any(|a| a.path().is_ident(ENTITIES))
                        || relationship.is_some_and(|relationship| relationship == *field)
                })
                .for_each(|(index, field)| {
                    let field_member =
                        field.ident.clone().map_or(Member::from(index), Member::Named);

                    map.push(quote!(#self_ident.#field_member.map_entities(mapper);));
                });
            if map.is_empty() {
                return None;
            };
            Some(quote!(
                #(#map)*
            ))
        }
        Data::Enum(DataEnum {
            variants,
            ..
        }) => {
            let mut map = Vec::with_capacity(variants.len());

            for variant in variants.iter() {
                let field_members = variant
                    .fields
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| field.attrs.iter().any(|a| a.path().is_ident(ENTITIES)))
                    .map(|(index, field)| {
                        field.ident.clone().map_or(Member::from(index), Member::Named)
                    })
                    .collect::<Vec<_>>();

                let ident = &variant.ident;
                let field_idents = field_members
                    .iter()
                    .map(|member| format_ident!("__self_{}", member))
                    .collect::<Vec<_>>();

                map.push(quote!(Self::#ident {#(#field_members: #field_idents,)* ..} => {
                    #(#field_idents.map_entities(mapper);)*
                }));
            }

            if map.is_empty() {
                return None;
            };

            Some(quote!(
                match #self_ident {
                    #(#map,)*
                    _ => {}
                }
            ))
        }
        Data::Union(_) => None,
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
    fn test_derive_component() {
        let expected = indoc! {r#"
          impl obel_ecs::component::Component for MyComponent
          where
              Self: Send + Sync + 'static,
          {
              const STORAGE_TYPE: obel_ecs::component::StorageType = obel_ecs::component::StorageType::Table;
              type Mutability = obel_ecs::component::Mutable;
              fn register_required_components(
                  requiree: obel_ecs::component::ComponentId,
                  components: &mut obel_ecs::component::ComponentsRegistrator,
                  required_components: &mut obel_ecs::component::RequiredComponents,
                  inheritance_depth: u16,
                  recursion_check_stack: &mut obel_ecs::__macro_exports::Vec<
                      obel_ecs::component::ComponentId,
                  >,
              ) {
                  obel_ecs::component::enforce_no_required_components_recursion(
                      components,
                      recursion_check_stack,
                  );
                  let self_id = components.register_component::<Self>();
                  recursion_check_stack.push(self_id);
                  recursion_check_stack.pop();
              }
              fn on_add() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                  ::core::option::Option::Some(view::add_visibility_class::<LightVisibilityClass>)
              }
              fn on_insert() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                  ::core::option::Option::Some(ord_a_hook_on_insert)
              }
              fn on_replace() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                  ::core::option::Option::Some(ord_a_hook_on_replace)
              }
              fn on_remove() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                  ::core::option::Option::Some(ord_a_hook_on_remove)
              }
              fn clone_behavior() -> obel_ecs::component::ComponentCloneBehavior {
                  use obel_ecs::component::{
                      DefaultCloneBehaviorBase, DefaultCloneBehaviorViaClone,
                  };
                  (&&&obel_ecs::component::DefaultCloneBehaviorSpecialization::<Self>::default())
                      .default_clone_behavior()
              }
          }
        "#};

        let actual = derive_component_impl(quote! {
            #[derive(Component)]
            #[component(
            storage = "Table",
            on_add = view::add_visibility_class::<LightVisibilityClass>,
            on_insert = ord_a_hook_on_insert,
            on_replace = ord_a_hook_on_replace,
            on_remove = ord_a_hook_on_remove)]
            struct MyComponent;
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_component_relationship() {
        let expected = indoc! {r#"
        #[doc = "**Required Components**: [`ColorGrading`], [`Exposure`]. \n\n A component's Required Components are inserted whenever it is inserted. Note that this will also insert the required components _of_ the required components, recursively, in depth-first order."]
        impl obel_ecs::component::Component for ChildOf
        where
            Self: Send + Sync + 'static,
        {
            const STORAGE_TYPE: obel_ecs::component::StorageType = obel_ecs::component::StorageType::Table;
            type Mutability = obel_ecs::component::Immutable;
            fn register_required_components(
                requiree: obel_ecs::component::ComponentId,
                components: &mut obel_ecs::component::ComponentsRegistrator,
                required_components: &mut obel_ecs::component::RequiredComponents,
                inheritance_depth: u16,
                recursion_check_stack: &mut obel_ecs::__macro_exports::Vec<
                    obel_ecs::component::ComponentId,
                >,
            ) {
                obel_ecs::component::enforce_no_required_components_recursion(
                    components,
                    recursion_check_stack,
                );
                let self_id = components.register_component::<Self>();
                recursion_check_stack.push(self_id);
                components
                    .register_required_components_manual::<
                        Self,
                        ColorGrading,
                    >(
                        required_components,
                        <ColorGrading as Default>::default,
                        inheritance_depth,
                        recursion_check_stack,
                    );
                components
                    .register_required_components_manual::<
                        Self,
                        Exposure,
                    >(
                        required_components,
                        <Exposure as Default>::default,
                        inheritance_depth,
                        recursion_check_stack,
                    );
                <ColorGrading as obel_ecs::component::Component>::register_required_components(
                    requiree,
                    components,
                    required_components,
                    inheritance_depth + 1,
                    recursion_check_stack,
                );
                <Exposure as obel_ecs::component::Component>::register_required_components(
                    requiree,
                    components,
                    required_components,
                    inheritance_depth + 1,
                    recursion_check_stack,
                );
                recursion_check_stack.pop();
            }
            fn on_add() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(view::add_visibility_class::<LightVisibilityClass>)
            }
            fn on_insert() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(
                    <Self as obel_ecs::relationship::Relationship>::on_insert,
                )
            }
            fn on_replace() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(
                    <Self as obel_ecs::relationship::Relationship>::on_replace,
                )
            }
            fn on_remove() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(ord_a_hook_on_remove)
            }
            fn clone_behavior() -> obel_ecs::component::ComponentCloneBehavior {
                use obel_ecs::component::{
                    DefaultCloneBehaviorBase, DefaultCloneBehaviorViaClone,
                };
                (&&&obel_ecs::component::DefaultCloneBehaviorSpecialization::<Self>::default())
                    .default_clone_behavior()
            }
            fn map_entities<M: obel_ecs::entity::EntityMapper>(this: &mut Self, mapper: &mut M) {
                use obel_ecs::entity::MapEntities;
                this.parent.map_entities(mapper);
                this.a.map_entities(mapper);
            }
        }
        impl obel_ecs::relationship::Relationship for ChildOf {
            type RelationshipTarget = Children;
            #[inline(always)]
            fn get(&self) -> obel_ecs::entity::Entity {
                self.parent
            }
            #[inline]
            fn from(entity: obel_ecs::entity::Entity) -> Self {
                Self {
                    a: core::default::Default::default(),
                    parent: collection,
                }
            }
        }
        "#};

        let actual = derive_component_impl(quote! {
            #[derive(Component)]
            #[component(
              storage = "Table",
              on_add = view::add_visibility_class::<LightVisibilityClass>,
              // on_replace = ord_a_hook_on_replace,
              // on_insert = ord_a_hook_on_insert,
              on_remove = ord_a_hook_on_remove
            )]
            #[relationship(relationship_target = Children)]
            #[require(ColorGrading, Exposure)]
            pub struct ChildOf {
                #[relationship]
                pub parent: Entity,
                #[entities]
                a: Entity,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_component_relationship_target() {
        let expected = indoc! {r#"
        #[doc = "**Required Components**: [`Camera`], [`DebandDither`]. \n\n A component's Required Components are inserted whenever it is inserted. Note that this will also insert the required components _of_ the required components, recursively, in depth-first order."]
        impl obel_ecs::component::Component for Children
        where
            Self: Send + Sync + 'static,
        {
            const STORAGE_TYPE: obel_ecs::component::StorageType = obel_ecs::component::StorageType::Table;
            type Mutability = obel_ecs::component::Mutable;
            fn register_required_components(
                requiree: obel_ecs::component::ComponentId,
                components: &mut obel_ecs::component::ComponentsRegistrator,
                required_components: &mut obel_ecs::component::RequiredComponents,
                inheritance_depth: u16,
                recursion_check_stack: &mut obel_ecs::__macro_exports::Vec<
                    obel_ecs::component::ComponentId,
                >,
            ) {
                obel_ecs::component::enforce_no_required_components_recursion(
                    components,
                    recursion_check_stack,
                );
                let self_id = components.register_component::<Self>();
                recursion_check_stack.push(self_id);
                components
                    .register_required_components_manual::<
                        Self,
                        Camera,
                    >(
                        required_components,
                        <Camera as Default>::default,
                        inheritance_depth,
                        recursion_check_stack,
                    );
                components
                    .register_required_components_manual::<
                        Self,
                        DebandDither,
                    >(
                        required_components,
                        || {
                            let x: DebandDither = (|| DebandDither(|| DebandDither::Enabled))()
                                .into();
                            x
                        },
                        inheritance_depth,
                        recursion_check_stack,
                    );
                <Camera as obel_ecs::component::Component>::register_required_components(
                    requiree,
                    components,
                    required_components,
                    inheritance_depth + 1,
                    recursion_check_stack,
                );
                <DebandDither as obel_ecs::component::Component>::register_required_components(
                    requiree,
                    components,
                    required_components,
                    inheritance_depth + 1,
                    recursion_check_stack,
                );
                recursion_check_stack.pop();
            }
            fn on_add() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(view::add_visibility_class::<LightVisibilityClass>)
            }
            fn on_insert() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(ord_a_hook_on_insert)
            }
            fn on_replace() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(
                    <Self as obel_ecs::relationship::RelationshipTarget>::on_replace,
                )
            }
            fn on_remove() -> ::core::option::Option<obel_ecs::component::ComponentHook> {
                ::core::option::Option::Some(ord_a_hook_on_remove)
            }
            fn clone_behavior() -> obel_ecs::component::ComponentCloneBehavior {
                obel_ecs::component::ComponentCloneBehavior::Custom(
                    obel_ecs::relationship::clone_relationship_target::<Self>,
                )
            }
            fn map_entities<M: obel_ecs::entity::EntityMapper>(this: &mut Self, mapper: &mut M) {
                use obel_ecs::entity::MapEntities;
                this.0.map_entities(mapper);
            }
        }
        impl obel_ecs::relationship::RelationshipTarget for Children {
            const LINKED_SPAWN: bool = false;
            type Relationship = ChildOf;
            type Collection = Vec<Entity>;
            #[inline]
            fn collection(&self) -> &Self::Collection {
                &self.0
            }
            #[inline]
            fn collection_mut_risky(&mut self) -> &mut Self::Collection {
                &mut self.0
            }
            #[inline]
            fn from_collection_risky(collection: Self::Collection) -> Self {
                Self { 0: collection }
            }
        }
        "#};

        let actual = derive_component_impl(quote! {
            #[derive(Component)]
            #[component(
              storage = "Table",
              on_add = view::add_visibility_class::<LightVisibilityClass>,
              // on_replace = ord_a_hook_on_replace,
              on_insert = ord_a_hook_on_insert,
              on_remove = ord_a_hook_on_remove
            )]
            #[relationship_target(relationship = ChildOf)]
            #[require(Camera, DebandDither(|| DebandDither::Enabled))]
            pub struct Children(Vec<Entity>);
        });

        assert_formatted_eq(actual, expected);
    }
}
