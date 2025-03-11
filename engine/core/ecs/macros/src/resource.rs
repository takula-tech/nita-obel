use crate::obel_ecs_path;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_quote, parse2};

pub fn derive_resource_impl(input: TokenStream) -> TokenStream {
    let obel_ecs_path = obel_ecs_path();
    let mut ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    ast.generics.make_where_clause().predicates.push(parse_quote! { Self: Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    quote! {
        impl #impl_generics #obel_ecs_path::resource::Resource for #struct_name #type_generics #where_clause {
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::string::ToString;

    #[test]
    fn test_derive_resource() {
        assert_eq!(
            derive_resource_impl(quote! {
                struct MyResource;
            })
            .to_string(),
            quote! {
              impl obel_ecs::resource::Resource for MyResource where Self : Send + Sync + 'static {
              }
          }.to_string()
        );
    }
}
