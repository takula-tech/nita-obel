use syn::{
    ExprClosure, Path, Result, parenthesized,
    parse::{Parse, ParseStream},
    token::Paren,
};

pub enum RequireFunc {
    Path(Path),
    Closure(ExprClosure),
}

pub struct Require {
    pub path: Path,
    pub func: Option<RequireFunc>,
}

impl Parse for Require {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse::<Path>()?;
        let func = if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            if let Ok(func) = content.parse::<ExprClosure>() {
                Some(RequireFunc::Closure(func))
            } else {
                let func = content.parse::<Path>()?;
                Some(RequireFunc::Path(func))
            }
        } else {
            None
        };
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
        if let Some(RequireFunc::Path(func_path)) = require.func {
            let func_str = quote!(#func_path).to_string();
            assert_eq!(func_str, "validate_path");
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
        // Verify the function is a Closure
        assert!(matches!(require.func, Some(RequireFunc::Closure(_))));
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
