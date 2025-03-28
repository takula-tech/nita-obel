use alloc::string::ToString;
use proc_macro2::TokenStream;
use syn::{
    Expr, Path, Result, Token, braced, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Brace, Paren},
};

pub struct Require {
    pub path: Path,
    pub func: Option<TokenStream>,
}

impl Parse for Require {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut path = input.parse::<Path>()?;
        let mut last_segment_is_lower = false;
        let mut is_constructor_call = false;
        // Use the case of the type name to check if it's an enum
        // This doesn't match everything that can be an enum according to the rust spec
        // but it matches what clippy is OK with
        let is_enum = {
            let mut first_chars =
                path.segments.iter().rev().filter_map(|s| s.ident.to_string().chars().next());
            if let Some(last) = first_chars.next() {
                if last.is_uppercase() {
                    if let Some(last) = first_chars.next() {
                        last.is_uppercase()
                    } else {
                        false
                    }
                } else {
                    last_segment_is_lower = true;
                    false
                }
            } else {
                false
            }
        };

        let func = if input.peek(Token![=]) {
            // If there is an '=', then this is a "function style" require
            let _t: syn::Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            let tokens: TokenStream = quote::quote! (|| #expr);
            Some(tokens)
        } else if input.peek(Brace) {
            // This is a "value style" named-struct-like require
            let content;
            braced!(content in input);
            let content = content.parse::<TokenStream>()?;
            let tokens: TokenStream = quote::quote! (|| #path { #content });
            Some(tokens)
        } else if input.peek(Paren) {
            // This is a "value style" tuple-struct-like require
            let content;
            parenthesized!(content in input);
            let content = content.parse::<TokenStream>()?;
            is_constructor_call = last_segment_is_lower;
            let tokens: TokenStream = quote::quote! (|| #path (#content));
            Some(tokens)
        } else if is_enum {
            // if this is an enum, then it is an inline enum component declaration
            let tokens: TokenStream = quote::quote! (|| #path);
            Some(tokens)
        } else {
            // if this isn't any of the above, then it is a component ident, which will use Default
            None
        };

        if is_enum || is_constructor_call {
            let path_len = path.segments.len();
            path = Path {
                leading_colon: path.leading_colon,
                segments: Punctuated::from_iter(path.segments.into_iter().take(path_len - 1)),
            };
        }
        Ok(Require {
            path,
            func,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::string::ToString;
    use syn::parse_quote;

    #[test]
    fn test_parse_path_only() {
        // Test parsing a simple path without a function
        let require: Require = parse_quote!(std::path::Path);
        assert!(matches!(require.path, Path { .. }));
        assert!(require.func.is_none());
        // Verify the path is correct
        let require_path = require.path;
        assert_eq!(quote!(#require_path).to_string(), "std :: path :: Path");
    }

    #[test]
    fn test_parse_with_path_func() {
        // Test parsing a path with a path function
        let require: Require = parse_quote!(std::path::Path(validate_path));
        assert!(matches!(require.path, Path { .. }));
        assert!(require.func.is_some());
        // Verify the function is a Path
        if let Some(func_path) = require.func {
            let func_str = func_path.to_string();
            assert_eq!(func_str, "|| std :: path :: Path (validate_path)");
        } else {
            panic!("Expected RequireFunc::Path");
        }
    }

    #[test]
    fn test_parse_with_closure_func() {
        // Test parsing a path with a closure function
        let require: Require = parse_quote!(std::path::Path(|x| x.is_valid()));
        assert!(matches!(require.path, Path { .. }));
        assert!(require.func.is_some());
    }

    #[test]
    fn test_parse_complex_path() {
        // Test parsing a complex path with segments and generics
        let require: Require = parse_quote!(std::collections::HashMap<String, u32>);
        assert!(matches!(require.path, Path { .. }));
        assert!(require.func.is_none());
        // Verify the path contains the expected components
        let require_path = require.path;
        let path_str = quote!(#require_path).to_string();
        assert_eq!(path_str, "std :: collections :: HashMap < String , u32 >");
    }
}
