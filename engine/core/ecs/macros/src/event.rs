use crate::obel_ecs_path;
use proc_macro2::TokenStream;
use quote::quote;
use std::format;
use syn::{DeriveInput, Type, parse_quote, parse2};

pub const EVENT: &str = "event";
pub const AUTO_PROPAGATE: &str = "auto_propagate";
pub const TRAVERSAL: &str = "traversal";

pub fn derive_event_impl(input: TokenStream) -> TokenStream {
    let obel_ecs_path = obel_ecs_path();
    let mut ast = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };

    let mut auto_propagate = false;
    let mut traversal: Type = parse_quote!(());

    ast.generics.make_where_clause().predicates.push(parse_quote! { Self: Send + Sync + 'static });

    if let Some(attr) = ast.attrs.iter().find(|attr| attr.path().is_ident(EVENT)) {
        if let Err(e) = attr.parse_nested_meta(|meta| match meta.path.get_ident() {
            Some(ident) if ident == AUTO_PROPAGATE => {
                auto_propagate = true;
                Ok(())
            }
            Some(ident) if ident == TRAVERSAL => {
                traversal = meta.value()?.parse()?;
                Ok(())
            }
            Some(ident) => Err(meta.error(format!("unsupported attribute: {}", ident))),
            None => Err(meta.error("expected identifier")),
        }) {
            return e.to_compile_error();
        }
    }

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    quote! {
        impl #impl_generics #obel_ecs_path::event::Event for #struct_name #type_generics #where_clause {
            type Traversal = #traversal;
            const AUTO_PROPAGATE: bool = #auto_propagate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::string::ToString;
    use syn::parse_quote;

    #[test]
    fn test_derive_event() {
        let input = parse_quote! {
            struct MyEvent;
        };
        let output = derive_event_impl(input);
        let expected = quote! {
            impl obel_ecs::event::Event for MyEvent where Self : Send + Sync + 'static {
                type Traversal = ();
                const AUTO_PROPAGATE: bool = false;
            }
        };
        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_derive_event_auto_propagate_and_traversal() {
        let input = quote! {
            #[event(auto_propagate, traversal = MyTraversal)]
            struct MyEvent;
        };
        let expected_output = quote! {
            impl obel_ecs::event::Event for MyEvent where Self : Send + Sync + 'static {
                type Traversal = MyTraversal;
                const AUTO_PROPAGATE: bool = true;
            }
        };
        let output = derive_event_impl(input);
        assert_eq!(output.to_string(), expected_output.to_string());
    }
}
