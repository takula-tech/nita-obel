use obel_reflect_utils::get_struct_fields;
use proc_macro2::TokenStream;
use quote::quote;
use std::{format, vec::Vec};
use syn::{DeriveInput, Index, parse2, spanned::Spanned};

use crate::obel_ecs_path;

pub fn derive_visit_entities_mut_impl(input: TokenStream) -> TokenStream {
    derive_visit_entities_base(input, quote! { VisitEntitiesMut }, |field| {
        quote! {
            fn visit_entities_mut<F: FnMut(&mut Entity)>(&mut self, mut f: F) {
                #(#field.visit_entities_mut(&mut f);)*
            }
        }
    })
}

pub fn derive_visit_entities_impl(input: TokenStream) -> TokenStream {
    derive_visit_entities_base(input, quote! { VisitEntities }, |field| {
        quote! {
            fn visit_entities<F: FnMut(Entity)>(&self, mut f: F) {
                #(#field.visit_entities(&mut f);)*
            }
        }
    })
}

fn derive_visit_entities_base(
    input: TokenStream,
    trait_name: TokenStream,
    gen_methods: impl FnOnce(Vec<TokenStream>) -> TokenStream,
) -> TokenStream {
    let ecs_path = obel_ecs_path();
    let ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let named_fields = match get_struct_fields(&ast.data) {
        Ok(fields) => fields,
        Err(e) => return e.into_compile_error(),
    };

    let field = named_fields
        .iter()
        .filter_map(|field| {
            if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("visit_entities")) {
                let ignore = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("ignore") {
                        Ok(())
                    } else {
                        Err(meta.error("Invalid visit_entities attribute. Use `ignore`"))
                    }
                });
                return match ignore {
                    Ok(()) => None,
                    Err(e) => Some(Err(e)),
                };
            }
            Some(Ok(field))
        })
        .map(|res| res.map(|field| field.ident.as_ref()))
        .collect::<Result<Vec<_>, _>>();

    let field = match field {
        Ok(field) => field,
        Err(e) => return e.into_compile_error(),
    };

    if field.is_empty() {
        return syn::Error::new(
            ast.span(),
            format!("Invalid `{}` type: at least one field", trait_name),
        )
        .into_compile_error();
    }

    let field_access = field
        .iter()
        .enumerate()
        .map(|(n, f)| {
            if let Some(ident) = f {
                quote! {
                    self.#ident
                }
            } else {
                let idx = Index::from(n);
                quote! {
                    self.#idx
                }
            }
        })
        .collect::<Vec<_>>();

    let methods = gen_methods(field_access);

    let generics = ast.generics;
    let (impl_generics, ty_generics, _) = generics.split_for_impl();
    let struct_name = &ast.ident;

    quote! {
        impl #impl_generics #ecs_path::entity:: #trait_name for #struct_name #ty_generics {
            #methods
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
    fn test_derive_visit_entities_impl() {
        let expected = indoc! {r#"
          impl obel_ecs::entity::VisitEntities for MyStruct {
              fn visit_entities<F: FnMut(Entity)>(&self, mut f: F) {
                  self.field1.visit_entities(&mut f);
                  self.field3.visit_entities(&mut f);
              }
          }
        "#};

        let actual = derive_visit_entities_impl(quote! {
            #[derive(VisitEntities)]
            struct MyStruct {
                field1: Entity,
                #[visit_entities(ignore)]
                field2: Entity,
                field3: Entity,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_visit_entities_mut_impl() {
        let expected = indoc! {r#"
          impl obel_ecs::entity::VisitEntitiesMut for MyStruct {
              fn visit_entities_mut<F: FnMut(&mut Entity)>(&mut self, mut f: F) {
                  self.field1.visit_entities_mut(&mut f);
                  self.field3.visit_entities_mut(&mut f);
              }
          }
        "#};

        let actual = derive_visit_entities_mut_impl(quote! {
            #[derive(VisitEntities)]
            struct MyStruct {
                field1: Entity,
                #[visit_entities(ignore)]
                field2: Entity,
                field3: Entity,
            }
        });

        assert_formatted_eq(actual, expected);
    }

    #[test]
    fn test_derive_visit_entities_base_empty_fields() {
        assert!(
            derive_visit_entities_impl(quote! {
                struct MyStruct {}
            })
            .to_string()
            .contains("Invalid `VisitEntities` type: at least one field")
        );
    }

    #[test]
    fn test_derive_visit_entities_base_invalid_attribute() {
        assert!(
            derive_visit_entities_impl(quote! {
                struct MyStruct {
                    field1: Entity,
                    #[visit_entities(invalid)]
                    field2: Entity,
                }
            })
            .to_string()
            .contains("Invalid visit_entities attribute")
        );
    }
}
