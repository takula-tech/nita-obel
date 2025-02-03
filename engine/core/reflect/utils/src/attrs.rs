use crate::symbol::Symbol;
use obel_platform::string::format;
use syn::{Expr, ExprLit, Lit};

/// Get a [literal string](struct@syn::LitStr) from the provided [expression](Expr).
pub fn get_lit_str(attr_name: Symbol, value: &Expr) -> syn::Result<&syn::LitStr> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit),
        ..
    }) = &value
    {
        Ok(lit)
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("expected {attr_name} attribute to be a string: `{attr_name} = \"...\"`"),
        ))
    }
}

/// Get a [literal boolean](struct@syn::LitBool) from the provided [expression](Expr) as a [`bool`].
pub fn get_lit_bool(attr_name: Symbol, value: &Expr) -> syn::Result<bool> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Bool(lit),
        ..
    }) = &value
    {
        Ok(lit.value())
    } else {
        Err(syn::Error::new_spanned(
            value,
            format!("expected {attr_name} attribute to be a bool value, `true` or `false`: `{attr_name} = ...`"),
        ))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_get_lit_str_success() {
        let attr_name = Symbol("test");
        let expr: Expr = parse2(quote! { "hello" }).unwrap();
        let result = get_lit_str(attr_name, &expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), "hello");
    }

    #[test]
    fn test_get_lit_str_failure() {
        let attr_name = Symbol("test");
        let expr: Expr = parse2(quote! { 42 }).unwrap();
        let result = get_lit_str(attr_name, &expr);
        assert!(result.is_err());
        let _ = result.inspect_err(|e| {
            assert_eq!(e.to_string(), "expected test attribute to be a string: `test = \"...\"`");
        });
    }

    #[test]
    fn test_get_lit_bool_success() {
        let attr_name = Symbol("test");
        let expr: Expr = parse2(quote! { true }).unwrap();
        let result = get_lit_bool(attr_name, &expr);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let expr: Expr = parse2(quote! { false }).unwrap();
        let result = get_lit_bool(attr_name, &expr);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_get_lit_bool_failure() {
        let attr_name = Symbol("test");
        let expr: Expr = parse2(quote! { "true" }).unwrap();
        let result = get_lit_bool(attr_name, &expr);
        assert!(result.is_err());
        let _ = result.inspect_err(|e| {
            assert_eq!(
                e.to_string(),
                "expected test attribute to be a bool value, `true` or `false`: `test = ...`"
            );
        });
    }
}
