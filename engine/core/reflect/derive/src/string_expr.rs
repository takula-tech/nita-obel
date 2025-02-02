use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, LitStr};

/// Contains tokens representing different kinds of string.
#[derive(Clone)]
pub(crate) enum StringExpr {
    /// A string that is valid at compile time.
    ///
    /// This is either a string literal like `"mystring"`,
    /// or a string created by a macro like [`module_path`]
    /// or [`concat`].
    Const(TokenStream),
    /// A [string slice](str) that is borrowed for a `'static` lifetime.
    #[allow(dead_code)]
    Borrowed(TokenStream),
    /// An [owned string](String).
    Owned(TokenStream),
}

impl<T: ToString + Spanned> From<T> for StringExpr {
    fn from(value: T) -> Self {
        Self::from_lit(&LitStr::new(&value.to_string(), value.span()))
    }
}

impl StringExpr {
    /// Creates a [constant] [`StringExpr`] from a [`struct@LitStr`].
    ///
    /// [constant]: StringExpr::Const
    pub fn from_lit(lit: &LitStr) -> Self {
        Self::Const(lit.to_token_stream())
    }

    /// Creates a [constant] [`StringExpr`] by interpreting a [string slice][str] as a [`struct@LitStr`].
    ///
    /// [constant]: StringExpr::Const
    pub fn from_str(string: &str) -> Self {
        Self::Const(string.into_token_stream())
    }

    /// Returns tokens for an [owned string](String).
    ///
    /// The returned expression will allocate unless the [`StringExpr`] is [already owned].
    ///
    /// [already owned]: StringExpr::Owned
    pub fn into_owned(self) -> TokenStream {
        let obel_reflect_path = crate::meta::get_path_to_obel_reflect();

        match self {
            Self::Const(tokens) | Self::Borrowed(tokens) => quote! {
                #obel_reflect_path::__macro_exports::alloc_utils::ToString::to_string(#tokens)
            },
            Self::Owned(owned) => owned,
        }
    }

    /// Returns tokens for a statically borrowed [string slice](str).
    pub fn into_borrowed(self) -> TokenStream {
        match self {
            Self::Const(tokens) | Self::Borrowed(tokens) => tokens,
            Self::Owned(owned) => quote! {
                &#owned
            },
        }
    }

    /// Appends a [`StringExpr`] to another.
    ///
    /// If both expressions are [`StringExpr::Const`] this will use [`concat`] to merge them.
    pub fn appended_by(mut self, other: StringExpr) -> Self {
        if let Self::Const(tokens) = self {
            if let Self::Const(more) = other {
                return Self::Const(quote! {
                    ::core::concat!(#tokens, #more)
                });
            }
            self = Self::Const(tokens);
        }

        let owned = self.into_owned();
        let borrowed = other.into_borrowed();
        Self::Owned(quote! {
            #owned + #borrowed
        })
    }
}

impl Default for StringExpr {
    fn default() -> Self {
        StringExpr::from_str("")
    }
}

impl FromIterator<StringExpr> for StringExpr {
    fn from_iter<T: IntoIterator<Item = StringExpr>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(mut expr) => {
                for next in iter {
                    expr = expr.appended_by(next);
                }

                expr
            }
            None => Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_expr_conversion() {
        let const_expr = StringExpr::from_str("test");
        let owned_tokens = const_expr.clone().into_owned();
        let borrowed_tokens = const_expr.into_borrowed();

        assert!(owned_tokens.to_string().contains("to_string"));
        assert_eq!(borrowed_tokens.to_string(), "\"test\"");
    }

    #[test]
    fn test_string_expr_append() {
        let expr1 = StringExpr::from_str("hello");
        let expr2 = StringExpr::from_str(" world");

        // Test const + const concatenation
        let const_result = expr1.clone().appended_by(expr2.clone());
        assert!(const_result.into_borrowed().to_string().contains("concat"));

        // Test owned + borrowed concatenation
        let owned = StringExpr::Owned(quote! { String::from("hello") });
        let mixed_result = owned.appended_by(expr2);
        assert!(mixed_result.into_owned().to_string().contains("+"));
    }

    #[test]
    fn test_string_expr_from_iterator() {
        let exprs = vec![
            StringExpr::from_str("hello"),
            StringExpr::from_str(" "),
            StringExpr::from_str("world"),
        ];

        let result: StringExpr = exprs.into_iter().collect();
        assert!(result.into_borrowed().to_string().contains("concat"));

        // Test empty iterator
        let empty_result: StringExpr = Vec::<StringExpr>::new().into_iter().collect();
        assert_eq!(empty_result.into_borrowed().to_string(), "\"\"");
    }

    #[test]
    fn test_string_expr_default() {
        let default_expr = StringExpr::default();
        assert_eq!(default_expr.into_borrowed().to_string(), "\"\"");
    }
}
