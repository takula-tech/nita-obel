use crate::obel_ecs_path;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, parse2};

pub fn derive_from_world_impl(input: TokenStream) -> TokenStream {
    let obel_ecs_path = obel_ecs_path();
    let ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let name = ast.ident;
    let (impl_generics, ty_generics, where_clauses) = ast.generics.split_for_impl();

    let (fields, variant_ident) =
        match &ast.data {
            Data::Struct(data) => (&data.fields, None),
            Data::Enum(data) => {
                match data.variants.iter().find(|variant| {
                    variant.attrs.iter().any(|attr| attr.path().is_ident("from_world"))
                }) {
                    Some(variant) => (&variant.fields, Some(&variant.ident)),
                    None => {
                        return syn::Error::new(
                            Span::call_site(),
                            "No variant found with the `#[from_world]` attribute",
                        )
                        .into_compile_error();
                    }
                }
            }
            Data::Union(_) => {
                return syn::Error::new(
                    Span::call_site(),
                    "#[derive(FromWorld)]` does not support unions",
                )
                .into_compile_error();
            }
        };

    let field_init_expr = quote!(#obel_ecs_path::world::FromWorld::from_world(world));
    let members = fields.members();

    let field_initializers = match variant_ident {
        Some(variant_ident) => quote!( Self::#variant_ident {
            #(#members: #field_init_expr),*
        }),
        None => quote!( Self {
            #(#members: #field_init_expr),*
        }),
    };

    quote! {
            impl #impl_generics #obel_ecs_path::world::FromWorld for #name #ty_generics #where_clauses {
                fn from_world(world: &mut #obel_ecs_path::world::World) -> Self {
                    #field_initializers
                }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use quote::quote;
    use std::string::ToString;

    #[track_caller]
    fn assert_formatted_eq(actual: TokenStream, expected: &str) {
        let syntax_tree: syn::File = parse2(actual).unwrap();
        let pretty = prettyplease::unparse(&syntax_tree);
        assert_eq!(pretty, expected, "\n === Pretty Please ===\n{}", pretty);
    }

    #[test]
    fn test_derive_from_world_struct() {
        let expected = indoc! {r#"
            impl obel_ecs::world::FromWorld for MyStruct {
                fn from_world(world: &mut obel_ecs::world::World) -> Self {
                    Self {
                        field1: obel_ecs::world::FromWorld::from_world(world),
                        field2: obel_ecs::world::FromWorld::from_world(world),
                    }
                }
            }
        "#};

        let actual = derive_from_world_impl(quote! {
            #[derive(FromWorld)]
            struct MyStruct {
                field1: i32,
                field2: String,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_from_world_enum() {
        let expected = indoc! {r#"
            impl obel_ecs::world::FromWorld for MyEnum {
                fn from_world(world: &mut obel_ecs::world::World) -> Self {
                    Self::Variant1 {
                        field1: obel_ecs::world::FromWorld::from_world(world),
                        field2: obel_ecs::world::FromWorld::from_world(world),
                    }
                }
            }
        "#};

        let actual = derive_from_world_impl(quote! {
            #[derive(FromWorld)]
            enum MyEnum {
                #[from_world]
                Variant1 {
                    field1: i32,
                    field2: String,
                },
                Variant2,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_from_world_enum_no_from_world_variant() {
        assert!(
            derive_from_world_impl(quote! {
                #[derive(FromWorld)]
                enum MyEnum {
                    Variant1 {
                        field1: i32,
                        field2: String,
                    },
                    Variant2,
                }
            })
            .to_string()
            .contains("No variant found with the `#[from_world]` attribute")
        );
    }

    #[test]
    fn test_derive_from_world_union() {
        assert!(
            derive_from_world_impl(quote! {
                #[derive(FromWorld)]
                union MyUnion {
                    field1: i32,
                    field2: f32,
                }
            })
            .to_string()
            .contains("#[derive(FromWorld)]` does not support unions")
        );
    }

    #[test]
    fn test_derive_from_world_invalid_input() {
        assert!(
            derive_from_world_impl(quote! {
                invalid syntax
            })
            .to_string()
            .contains("expected one of: `struct`, `enum`, `union`")
        );
    }
}
