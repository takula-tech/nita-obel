use obel_reflect_utils::get_struct_fields;
use proc_macro2::TokenStream;
use quote::quote;
use std::{format, vec::Vec};
use syn::{DeriveInput, Index, parse2};

use crate::obel_ecs_path;

const BUNDLE_ATTRIBUTE_NAME: &str = "bundle";
const BUNDLE_ATTRIBUTE_IGNORE_NAME: &str = "ignore";

enum BundleFieldKind {
    Component,
    Ignore,
}

pub fn derive_bundle_impl(input: TokenStream) -> TokenStream {
    let ecs_path = obel_ecs_path();
    let ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let named_fields = match get_struct_fields(&ast.data) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error(),
    };

    let mut field_kind = Vec::with_capacity(named_fields.len());

    for field in named_fields {
        for attr in field.attrs.iter().filter(|a| a.path().is_ident(BUNDLE_ATTRIBUTE_NAME)) {
            if let Err(error) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(BUNDLE_ATTRIBUTE_IGNORE_NAME) {
                    field_kind.push(BundleFieldKind::Ignore);
                    Ok(())
                } else {
                    Err(meta.error(format!(
                        "Invalid bundle attribute. Use `{BUNDLE_ATTRIBUTE_IGNORE_NAME}`"
                    )))
                }
            }) {
                return error.into_compile_error();
            }
        }

        field_kind.push(BundleFieldKind::Component);
    }

    let field = named_fields.iter().map(|field| field.ident.as_ref()).collect::<Vec<_>>();

    let field_type = named_fields.iter().map(|field| &field.ty).collect::<Vec<_>>();

    let mut field_component_ids = Vec::new();
    let mut field_get_component_ids = Vec::new();
    let mut field_get_components = Vec::new();
    let mut field_from_components = Vec::new();
    let mut field_required_components = Vec::new();
    for (((i, field_type), field_kind), field) in
        field_type.iter().enumerate().zip(field_kind.iter()).zip(field.iter())
    {
        match field_kind {
            BundleFieldKind::Component => {
                field_component_ids.push(quote! {
                <#field_type as #ecs_path::bundle::Bundle>::component_ids(components, &mut *ids);
                });
                field_required_components.push(quote! {
                  <#field_type as #ecs_path::bundle::Bundle>::register_required_components(components, required_components);
              });
                field_get_component_ids.push(quote! {
                  <#field_type as #ecs_path::bundle::Bundle>::get_component_ids(components, &mut *ids);
              });
                match field {
                    Some(field) => {
                        field_get_components.push(quote! {
                            self.#field.get_components(&mut *func);
                        });
                        field_from_components.push(quote! {
                          #field: <#field_type as #ecs_path::bundle::BundleFromComponents>::from_components(ctx, &mut *func),
                      });
                    }
                    None => {
                        let index = Index::from(i);
                        field_get_components.push(quote! {
                            self.#index.get_components(&mut *func);
                        });
                        field_from_components.push(quote! {
                          #index: <#field_type as #ecs_path::bundle::BundleFromComponents>::from_components(ctx, &mut *func),
                      });
                    }
                }
            }

            BundleFieldKind::Ignore => {
                field_from_components.push(quote! {
                    #field: ::core::default::Default::default(),
                });
            }
        }
    }
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let struct_name = &ast.ident;

    quote! {
        // SAFETY:
        // - ComponentId is returned in field-definition-order. [get_components] uses field-definition-order
        // - `Bundle::get_components` is exactly once for each member. Rely's on the Component -> Bundle implementation to properly pass
        //   the correct `StorageType` into the callback.
        #[allow(deprecated)]
        unsafe impl #impl_generics #ecs_path::bundle::Bundle for #struct_name #ty_generics #where_clause {
            fn component_ids(
                components: &mut #ecs_path::component::ComponentsRegistrator,
                ids: &mut impl FnMut(#ecs_path::component::ComponentId)
            ){
                #(#field_component_ids)*
            }

            fn get_component_ids(
                components: &#ecs_path::component::Components,
                ids: &mut impl FnMut(Option<#ecs_path::component::ComponentId>)
            ){
                #(#field_get_component_ids)*
            }

            fn register_required_components(
                components: &mut #ecs_path::component::ComponentsRegistrator,
                required_components: &mut #ecs_path::component::RequiredComponents
            ){
                #(#field_required_components)*
            }
        }

        // SAFETY:
        // - ComponentId is returned in field-definition-order. [from_components] uses field-definition-order
        #[allow(deprecated)]
        unsafe impl #impl_generics #ecs_path::bundle::BundleFromComponents for #struct_name #ty_generics #where_clause {
            #[allow(unused_variables, non_snake_case)]
            unsafe fn from_components<__T, __F>(ctx: &mut __T, func: &mut __F) -> Self
            where
                __F: FnMut(&mut __T) -> #ecs_path::ptr::OwningPtr<'_>
            {
                Self{
                    #(#field_from_components)*
                }
            }
        }

        #[allow(deprecated)]
        impl #impl_generics #ecs_path::bundle::DynamicBundle for #struct_name #ty_generics #where_clause {
            type Effect = ();
            #[allow(unused_variables)]
            #[inline]
            fn get_components(
                self,
                func: &mut impl FnMut(#ecs_path::component::StorageType, #ecs_path::ptr::OwningPtr<'_>)
            ) {
                #(#field_get_components)*
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use proc_macro2::TokenStream;
    use quote::quote;
    use std::string::ToString;

    #[track_caller]
    fn assert_formatted_eq(actual: TokenStream, expected: &str) {
        let syntax_tree: syn::File = parse2(actual).unwrap();
        let pretty = prettyplease::unparse(&syntax_tree);
        assert_eq!(pretty, expected, "\n === Pretty Please ===\n{}", pretty);
    }

    #[test]
    fn test_derive_bundle_impl_simple_struct() {
        let expected = indoc! {r#"
            #[allow(deprecated)]
            unsafe impl obel_ecs::bundle::Bundle for MyStruct {
                fn component_ids(
                    components: &mut obel_ecs::component::ComponentsRegistrator,
                    ids: &mut impl FnMut(obel_ecs::component::ComponentId),
                ) {
                    <u32 as obel_ecs::bundle::Bundle>::component_ids(components, &mut *ids);
                    <String as obel_ecs::bundle::Bundle>::component_ids(components, &mut *ids);
                }
                fn get_component_ids(
                    components: &obel_ecs::component::Components,
                    ids: &mut impl FnMut(Option<obel_ecs::component::ComponentId>),
                ) {
                    <u32 as obel_ecs::bundle::Bundle>::get_component_ids(components, &mut *ids);
                    <String as obel_ecs::bundle::Bundle>::get_component_ids(components, &mut *ids);
                }
                fn register_required_components(
                    components: &mut obel_ecs::component::ComponentsRegistrator,
                    required_components: &mut obel_ecs::component::RequiredComponents,
                ) {
                    <u32 as obel_ecs::bundle::Bundle>::register_required_components(
                        components,
                        required_components,
                    );
                    <String as obel_ecs::bundle::Bundle>::register_required_components(
                        components,
                        required_components,
                    );
                }
            }
            #[allow(deprecated)]
            unsafe impl obel_ecs::bundle::BundleFromComponents for MyStruct {
                #[allow(unused_variables, non_snake_case)]
                unsafe fn from_components<__T, __F>(ctx: &mut __T, func: &mut __F) -> Self
                where
                    __F: FnMut(&mut __T) -> obel_ecs::ptr::OwningPtr<'_>,
                {
                    Self {
                        field1: <u32 as obel_ecs::bundle::BundleFromComponents>::from_components(
                            ctx,
                            &mut *func,
                        ),
                        field2: <String as obel_ecs::bundle::BundleFromComponents>::from_components(
                            ctx,
                            &mut *func,
                        ),
                    }
                }
            }
            #[allow(deprecated)]
            impl obel_ecs::bundle::DynamicBundle for MyStruct {
                type Effect = ();
                #[allow(unused_variables)]
                #[inline]
                fn get_components(
                    self,
                    func: &mut impl FnMut(
                        obel_ecs::component::StorageType,
                        obel_ecs::ptr::OwningPtr<'_>,
                    ),
                ) {
                    self.field1.get_components(&mut *func);
                    self.field2.get_components(&mut *func);
                }
            }
        "#};

        let actual = derive_bundle_impl(quote! {
            struct MyStruct {
                field1: u32,
                field2: String,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_bundle_impl_with_ignore_attribute() {
        let expected = indoc! {r#"
            #[allow(deprecated)]
            unsafe impl obel_ecs::bundle::Bundle for MyStruct {
                fn component_ids(
                    components: &mut obel_ecs::component::ComponentsRegistrator,
                    ids: &mut impl FnMut(obel_ecs::component::ComponentId),
                ) {
                    <u32 as obel_ecs::bundle::Bundle>::component_ids(components, &mut *ids);
                }
                fn get_component_ids(
                    components: &obel_ecs::component::Components,
                    ids: &mut impl FnMut(Option<obel_ecs::component::ComponentId>),
                ) {
                    <u32 as obel_ecs::bundle::Bundle>::get_component_ids(components, &mut *ids);
                }
                fn register_required_components(
                    components: &mut obel_ecs::component::ComponentsRegistrator,
                    required_components: &mut obel_ecs::component::RequiredComponents,
                ) {
                    <u32 as obel_ecs::bundle::Bundle>::register_required_components(
                        components,
                        required_components,
                    );
                }
            }
            #[allow(deprecated)]
            unsafe impl obel_ecs::bundle::BundleFromComponents for MyStruct {
                #[allow(unused_variables, non_snake_case)]
                unsafe fn from_components<__T, __F>(ctx: &mut __T, func: &mut __F) -> Self
                where
                    __F: FnMut(&mut __T) -> obel_ecs::ptr::OwningPtr<'_>,
                {
                    Self {
                        field1: <u32 as obel_ecs::bundle::BundleFromComponents>::from_components(
                            ctx,
                            &mut *func,
                        ),
                        field2: ::core::default::Default::default(),
                    }
                }
            }
            #[allow(deprecated)]
            impl obel_ecs::bundle::DynamicBundle for MyStruct {
                type Effect = ();
                #[allow(unused_variables)]
                #[inline]
                fn get_components(
                    self,
                    func: &mut impl FnMut(
                        obel_ecs::component::StorageType,
                        obel_ecs::ptr::OwningPtr<'_>,
                    ),
                ) {
                    self.field1.get_components(&mut *func);
                }
            }
        "#};

        let actual = derive_bundle_impl(quote! {
            struct MyStruct {
                field1: u32,
                #[bundle(ignore)]
                field2: String,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_bundle_impl_with_invalid_attribute() {
        assert!(
            derive_bundle_impl(quote! {
                struct MyStruct {
                    field1: u32,
                    #[bundle(invalid)]
                    field2: String,
                }
            })
            .to_string()
            .contains("Invalid bundle attribute")
        );
    }

    #[test]
    fn test_derive_bundle_impl_with_generics() {
        let expected = indoc! {r#"
            #[allow(deprecated)]
            unsafe impl<T> obel_ecs::bundle::Bundle for MyStruct<T> {
                fn component_ids(
                    components: &mut obel_ecs::component::ComponentsRegistrator,
                    ids: &mut impl FnMut(obel_ecs::component::ComponentId),
                ) {
                    <T as obel_ecs::bundle::Bundle>::component_ids(components, &mut *ids);
                    <String as obel_ecs::bundle::Bundle>::component_ids(components, &mut *ids);
                }
                fn get_component_ids(
                    components: &obel_ecs::component::Components,
                    ids: &mut impl FnMut(Option<obel_ecs::component::ComponentId>),
                ) {
                    <T as obel_ecs::bundle::Bundle>::get_component_ids(components, &mut *ids);
                    <String as obel_ecs::bundle::Bundle>::get_component_ids(components, &mut *ids);
                }
                fn register_required_components(
                    components: &mut obel_ecs::component::ComponentsRegistrator,
                    required_components: &mut obel_ecs::component::RequiredComponents,
                ) {
                    <T as obel_ecs::bundle::Bundle>::register_required_components(
                        components,
                        required_components,
                    );
                    <String as obel_ecs::bundle::Bundle>::register_required_components(
                        components,
                        required_components,
                    );
                }
            }
            #[allow(deprecated)]
            unsafe impl<T> obel_ecs::bundle::BundleFromComponents for MyStruct<T> {
                #[allow(unused_variables, non_snake_case)]
                unsafe fn from_components<__T, __F>(ctx: &mut __T, func: &mut __F) -> Self
                where
                    __F: FnMut(&mut __T) -> obel_ecs::ptr::OwningPtr<'_>,
                {
                    Self {
                        field1: <T as obel_ecs::bundle::BundleFromComponents>::from_components(
                            ctx,
                            &mut *func,
                        ),
                        field2: <String as obel_ecs::bundle::BundleFromComponents>::from_components(
                            ctx,
                            &mut *func,
                        ),
                    }
                }
            }
            #[allow(deprecated)]
            impl<T> obel_ecs::bundle::DynamicBundle for MyStruct<T> {
                type Effect = ();
                #[allow(unused_variables)]
                #[inline]
                fn get_components(
                    self,
                    func: &mut impl FnMut(
                        obel_ecs::component::StorageType,
                        obel_ecs::ptr::OwningPtr<'_>,
                    ),
                ) {
                    self.field1.get_components(&mut *func);
                    self.field2.get_components(&mut *func);
                }
            }
        "#};

        let actual = derive_bundle_impl(quote! {
            struct MyStruct<T> {
                field1: T,
                field2: String,
            }
        });

        assert_formatted_eq(actual, expected);
    }
}
