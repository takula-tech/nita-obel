use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Path, Token, parse::ParseStream};

#[derive(Default, Clone)]
pub(crate) struct CustomAttributes {
    attributes: Vec<Expr>,
}

impl CustomAttributes {
    /// Generates a `TokenStream` for `CustomAttributes` construction.
    pub fn to_tokens(&self, obel_reflect_path: &Path) -> TokenStream {
        let attributes = self.attributes.iter().map(|value| {
            quote! {
                .with_attribute(#value)
            }
        });

        quote! {
            #obel_reflect_path::attributes::CustomAttributes::default()
                #(#attributes)*
        }
    }

    /// Inserts a custom attribute into the list.
    pub fn push(&mut self, value: Expr) -> syn::Result<()> {
        self.attributes.push(value);
        Ok(())
    }

    /// Parse `@` (custom attribute) attribute.
    ///
    /// Examples:
    /// - `#[reflect(@Foo))]`
    /// - `#[reflect(@Bar::baz("qux"))]`
    /// - `#[reflect(@0..256u8)]`
    pub fn parse_custom_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<Token![@]>()?;
        self.push(input.parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, quote};
    use syn::{Expr, Path, parse_quote};

    #[test]
    fn test_push() {
        let mut custom_attrs = CustomAttributes::default();
        let expr: Expr = parse_quote!(Foo);
        custom_attrs.push(expr.clone()).unwrap();

        assert_eq!(custom_attrs.attributes.len(), 1);
        assert_eq!(
            expr.to_token_stream().to_string(),
            custom_attrs.attributes[0].to_token_stream().to_string()
        );
    }

    #[test]
    fn test_to_tokens() {
        let mut custom_attrs = CustomAttributes::default();
        let expr1: Expr = parse_quote!(Foo);
        let expr2: Expr = parse_quote!(Bar::baz("qux"));
        custom_attrs.push(expr1.clone()).unwrap();
        custom_attrs.push(expr2.clone()).unwrap();

        let obel_reflect_path: Path = parse_quote!(obel::reflect);
        let tokens = custom_attrs.to_tokens(&obel_reflect_path);

        let expected_tokens = quote! {
            obel::reflect::attributes::CustomAttributes::default()
                .with_attribute(#expr1)
                .with_attribute(#expr2)
        };

        assert_eq!(tokens.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn test_parse_custom_attribute() {
        let mut custom_attrs = CustomAttributes::default();
        let input: TokenStream = parse_quote!(@Foo);

        // Create a ParseStream from the input TokenStream
        let result = syn::parse::Parser::parse2(
            |input: ParseStream| custom_attrs.parse_custom_attribute(input),
            input,
        );

        assert!(result.is_ok());
        assert_eq!(custom_attrs.attributes.len(), 1);
        assert_eq!(
            custom_attrs.attributes[0].to_token_stream().to_string(),
            quote!(Foo).to_string()
        );
    }

    #[test]
    fn test_parse_custom_attribute_complex() {
        let mut custom_attrs = CustomAttributes::default();
        let input: TokenStream = quote!(@Bar::baz("qux"));

        // Create a ParseStream from the input TokenStream
        let result = syn::parse::Parser::parse2(
            |input: ParseStream| custom_attrs.parse_custom_attribute(input),
            input,
        );

        assert!(result.is_ok());
        assert_eq!(custom_attrs.attributes.len(), 1);
        assert_eq!(
            custom_attrs.attributes[0].to_token_stream().to_string(),
            quote!(Bar::baz("qux")).to_string()
        );
    }

    #[test]
    fn test_parse_custom_attribute_invalid() {
        let mut custom_attrs = CustomAttributes::default();
        let input: TokenStream = quote!(Foo); // Missing `@`
        let result = syn::parse::Parser::parse2(
            |input: ParseStream| custom_attrs.parse_custom_attribute(input),
            input,
        );
        assert!(result.is_err());
    }
}
