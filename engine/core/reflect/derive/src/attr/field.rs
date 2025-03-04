//! Contains code related to field attributes for reflected types.
//!
//! A field attribute is an attribute which applies to particular field or variant
//! as opposed to an entire struct or enum. An example of such an attribute is
//! the derive helper attribute for `Reflect`, which looks like: `#[reflect(ignore)]`.

use crate::{REFLECT_ATTRIBUTE_NAME, attr::CustomAttributes, attr::terminated_parser};
use quote::ToTokens;
use syn::{Attribute, LitStr, Meta, Token, Type, parse::ParseStream};

mod kw {
    syn::custom_keyword!(ignore);
    syn::custom_keyword!(skip_serializing);
    syn::custom_keyword!(default);
    syn::custom_keyword!(remote);
}

pub(crate) const IGNORE_SERIALIZATION_ATTR: &str = "skip_serializing";
pub(crate) const IGNORE_ALL_ATTR: &str = "ignore";
pub(crate) const DEFAULT_ATTR: &str = "default";

/// Stores data about if the field should be visible via the Reflect and serialization interfaces
///
/// Note the relationship between serialization and reflection is such that a member must be reflected in order to be serialized.
/// In boolean logic this is described as: `is_serialized -> is_reflected`, this means we can reflect something without serializing it but not the other way round.
/// The `is_reflected` predicate is provided as `self.is_active()`
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IgnoreBehavior {
    /// Don't ignore, appear to all systems
    #[default]
    None,
    /// Ignore when serializing but not when reflecting
    IgnoreSerialization,
    /// Ignore both when serializing and reflecting
    IgnoreAlways,
}

impl IgnoreBehavior {
    /// Returns `true` if the ignoring behavior implies member is included in the reflection API, and false otherwise.
    pub fn is_active(self) -> bool {
        match self {
            IgnoreBehavior::None | IgnoreBehavior::IgnoreSerialization => true,
            IgnoreBehavior::IgnoreAlways => false,
        }
    }

    /// The exact logical opposite of `self.is_active()` returns true iff this member is not part of the reflection API whatsoever (neither serialized nor reflected)
    pub fn is_ignored(self) -> bool {
        !self.is_active()
    }
}

/// Controls how the default value is determined for a field.
#[derive(Default, Clone)]
pub(crate) enum DefaultBehavior {
    /// Field is required.
    #[default]
    Required,
    /// Field can be defaulted using `Default::default()`.
    Default,
    /// Field can be created using the given function name.
    ///
    /// This assumes the function is in scope, is callable with zero arguments,
    /// and returns the expected type.
    Func(syn::ExprPath),
}

/// A container for attributes defined on a reflected type's field.
#[derive(Default, Clone)]
pub(crate) struct FieldAttributes {
    /// Determines how this field should be ignored if at all.
    pub ignore: IgnoreBehavior,
    /// Sets the default behavior of this field.
    pub default: DefaultBehavior,
    /// Custom attributes created via `#[reflect(@...)]`.
    pub custom_attributes: CustomAttributes,
    /// For defining the remote wrapper type that should be used in place of the field for reflection logic.
    pub remote: Option<Type>,
}

impl FieldAttributes {
    /// Parse all field attributes marked "reflect" (such as `#[reflect(ignore)]`).
    pub fn parse_attributes(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = FieldAttributes::default();

        attrs
            .iter()
            .filter_map(|attr| {
                if !attr.path().is_ident(REFLECT_ATTRIBUTE_NAME) {
                    // Not a reflect attribute -> skip
                    return None;
                }

                let Meta::List(meta) = &attr.meta else {
                    return Some(syn::Error::new_spanned(attr, "expected meta list"));
                };

                // Parse all attributes inside the list, collecting any errors
                meta.parse_args_with(terminated_parser(Token![,], |stream| {
                    args.parse_field_attribute(stream)
                }))
                .err()
            })
            .reduce(|mut acc, err| {
                acc.combine(err);
                acc
            })
            .map_or(Ok(args), Err)
    }

    /// Parses a single field attribute.
    fn parse_field_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![@]) {
            self.parse_custom_attribute(input)
        } else if lookahead.peek(kw::ignore) {
            self.parse_ignore(input)
        } else if lookahead.peek(kw::skip_serializing) {
            self.parse_skip_serializing(input)
        } else if lookahead.peek(kw::default) {
            self.parse_default(input)
        } else if lookahead.peek(kw::remote) {
            self.parse_remote(input)
        } else {
            Err(lookahead.error())
        }
    }

    /// Parse `ignore` attribute.
    ///
    /// Examples:
    /// - `#[reflect(ignore)]`
    fn parse_ignore(&mut self, input: ParseStream) -> syn::Result<()> {
        if self.ignore != IgnoreBehavior::None {
            return Err(input.error(format!(
                "only one of {:?} is allowed",
                [IGNORE_ALL_ATTR, IGNORE_SERIALIZATION_ATTR]
            )));
        }

        input.parse::<kw::ignore>()?;
        self.ignore = IgnoreBehavior::IgnoreAlways;
        Ok(())
    }

    /// Parse `skip_serializing` attribute.
    ///
    /// Examples:
    /// - `#[reflect(skip_serializing)]`
    fn parse_skip_serializing(&mut self, input: ParseStream) -> syn::Result<()> {
        if self.ignore != IgnoreBehavior::None {
            return Err(input.error(format!(
                "only one of {:?} is allowed",
                [IGNORE_ALL_ATTR, IGNORE_SERIALIZATION_ATTR]
            )));
        }

        input.parse::<kw::skip_serializing>()?;
        self.ignore = IgnoreBehavior::IgnoreSerialization;
        Ok(())
    }

    /// Parse `default` attribute.
    ///
    /// Examples:
    /// - `#[reflect(default)]`
    /// - `#[reflect(default = "path::to::func")]`
    fn parse_default(&mut self, input: ParseStream) -> syn::Result<()> {
        if !matches!(self.default, DefaultBehavior::Required) {
            return Err(input.error(format!("only one of {:?} is allowed", [DEFAULT_ATTR])));
        }

        input.parse::<kw::default>()?;

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            let lit = input.parse::<LitStr>()?;
            self.default = DefaultBehavior::Func(lit.parse()?);
        } else {
            self.default = DefaultBehavior::Default;
        }

        Ok(())
    }

    /// Parse `@` (custom attribute) attribute.
    ///
    /// Examples:
    /// - `#[reflect(@(foo = "bar"))]`
    /// - `#[reflect(@(min = 0.0, max = 1.0))]`
    fn parse_custom_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        self.custom_attributes.parse_custom_attribute(input)
    }

    /// Parse `remote` attribute.
    ///
    /// Examples:
    /// - `#[reflect(remote = path::to::RemoteType)]`
    fn parse_remote(&mut self, input: ParseStream) -> syn::Result<()> {
        if let Some(remote) = self.remote.as_ref() {
            return Err(input
                .error(format!("remote type already specified as {}", remote.to_token_stream())));
        }

        input.parse::<kw::remote>()?;
        input.parse::<Token![=]>()?;

        self.remote = Some(input.parse()?);

        Ok(())
    }

    /// Returns `Some(true)` if the field has a generic remote type.
    ///
    /// If the remote type is not generic, returns `Some(false)`.
    ///
    /// If the field does not have a remote type, returns `None`.
    pub fn is_remote_generic(&self) -> Option<bool> {
        if let Type::Path(type_path) = self.remote.as_ref()? {
            type_path.path.segments.last().map(|segment| !segment.arguments.is_empty())
        } else {
            Some(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    fn create_reflect_attribute(tokens: proc_macro2::TokenStream) -> Attribute {
        parse_quote!(#[reflect(#tokens)])
    }

    #[test]
    fn test_default_values() {
        let attrs = FieldAttributes::default();
        assert!(matches!(attrs.ignore, IgnoreBehavior::None));
        assert!(matches!(attrs.default, DefaultBehavior::Required));
        assert!(attrs.remote.is_none());
    }

    #[test]
    fn test_parse_ignore() {
        let attr = create_reflect_attribute(quote!(ignore));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        assert!(matches!(attrs.ignore, IgnoreBehavior::IgnoreAlways));
        assert!(attrs.is_remote_generic().is_none());
    }

    #[test]
    fn test_parse_skip_serializing() {
        let attr = create_reflect_attribute(quote!(skip_serializing));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        assert!(matches!(attrs.ignore, IgnoreBehavior::IgnoreSerialization));
    }

    #[test]
    fn test_parse_default() {
        let attr = create_reflect_attribute(quote!(default));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        assert!(matches!(attrs.default, DefaultBehavior::Default));
    }

    #[test]
    fn test_parse_default_with_function() {
        let attr = create_reflect_attribute(quote!(default = "my_module::create_default"));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        if let DefaultBehavior::Func(path) = &attrs.default {
            assert_eq!(path.to_token_stream().to_string(), "my_module :: create_default");
        } else {
            panic!("Expected DefaultBehavior::Func");
        }
    }

    #[test]
    fn test_parse_remote() {
        let attr = create_reflect_attribute(quote!(remote = Vec<String>));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        assert!(attrs.remote.is_some());
        assert_eq!(attrs.is_remote_generic(), Some(true));
    }

    #[test]
    fn test_parse_remote_non_generic() {
        let attr = create_reflect_attribute(quote!(remote = String));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        assert!(attrs.remote.is_some());
        assert_eq!(attrs.is_remote_generic(), Some(false));
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let attrs = FieldAttributes::parse_attributes(&[
            create_reflect_attribute(quote!(skip_serializing)),
            create_reflect_attribute(quote!(default)),
        ])
        .unwrap();

        assert!(matches!(attrs.ignore, IgnoreBehavior::IgnoreSerialization));
        assert!(matches!(attrs.default, DefaultBehavior::Default));
    }

    #[test]
    fn test_duplicate_ignore_attributes_error() {
        let result = FieldAttributes::parse_attributes(&[
            create_reflect_attribute(quote!(ignore)),
            create_reflect_attribute(quote!(skip_serializing)),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_default_attributes_error() {
        let result = FieldAttributes::parse_attributes(&[
            create_reflect_attribute(quote!(default)),
            create_reflect_attribute(quote!(default = "create_default")),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_remote_attributes_error() {
        let result = FieldAttributes::parse_attributes(&[
            create_reflect_attribute(quote!(remote = String)),
            create_reflect_attribute(quote!(remote = Vec<u32>)),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_ignore_behavior() {
        assert!(IgnoreBehavior::None.is_active());
        assert!(!IgnoreBehavior::None.is_ignored());

        assert!(IgnoreBehavior::IgnoreSerialization.is_active());
        assert!(!IgnoreBehavior::IgnoreSerialization.is_ignored());

        assert!(!IgnoreBehavior::IgnoreAlways.is_active());
        assert!(IgnoreBehavior::IgnoreAlways.is_ignored());
    }

    #[test]
    fn test_non_reflect_attribute_is_skipped() {
        let attr: Attribute = parse_quote!(#[derive(Debug)]);
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();
        assert!(matches!(attrs.ignore, IgnoreBehavior::None));
        assert!(matches!(attrs.default, DefaultBehavior::Required));
    }

    #[test]
    fn test_parse_multiple_values_in_single_attribute() {
        let attr = create_reflect_attribute(quote!(ignore, default));
        let attrs = FieldAttributes::parse_attributes(&[attr]).unwrap();

        // Verify both attributes were correctly parsed
        assert!(matches!(attrs.ignore, IgnoreBehavior::IgnoreAlways));
        assert!(matches!(attrs.default, DefaultBehavior::Default));
    }
}
