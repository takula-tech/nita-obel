//! Contains code related to container attributes for reflected types.
//!
//! A container attribute is an attribute which applies to an entire struct or enum
//! as opposed to a particular field or variant. An example of such an attribute is
//! the derive helper attribute for `Reflect`, which looks like:
//! `#[reflect(PartialEq, Default, ...)]`.

use crate::{attr::CustomAttributes, attr::terminated_parser, derive_data::ReflectTraitToImpl};
use obel_reflect_utils::{FQAny, FQOption};
use proc_macro2::{Ident, Span};
use quote::quote_spanned;
use syn::{
    Expr, LitBool, MetaList, MetaNameValue, Path, Token, WhereClause, ext::IdentExt, parenthesized,
    parse::ParseStream, spanned::Spanned, token,
};

mod kw {
    syn::custom_keyword!(from_reflect);
    syn::custom_keyword!(type_path);
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(PartialEq);
    syn::custom_keyword!(Hash);
    syn::custom_keyword!(no_field_bounds);
    syn::custom_keyword!(opaque);
}

// The "special" trait idents that are used internally for reflection.
// Received via attributes like `#[reflect(PartialEq, Hash, ...)]`
const DEBUG_ATTR: &str = "Debug";
const PARTIAL_EQ_ATTR: &str = "PartialEq";
const HASH_ATTR: &str = "Hash";

// The traits listed below are not considered "special" (i.e. they use the `ReflectMyTrait` syntax)
// but useful to know exist nonetheless
pub(crate) const REFLECT_DEFAULT: &str = "ReflectDefault";

// Attributes for `FromReflect` implementation
const FROM_REFLECT_ATTR: &str = "from_reflect";

// Attributes for `TypePath` implementation
const TYPE_PATH_ATTR: &str = "type_path";

// The error message to show when a trait/type is specified multiple times
const CONFLICTING_TYPE_DATA_MESSAGE: &str = "conflicting type data registration";

/// A marker for trait implementations registered via the `Reflect` derive macro.
#[derive(Clone, Default)]
pub(crate) enum TraitImpl {
    /// The trait is not registered as implemented.
    #[default]
    NotImplemented,

    /// The trait is registered as implemented.
    Implemented(Span),

    /// The trait is registered with a custom function rather than an actual implementation.
    Custom(Path, Span),
}

impl TraitImpl {
    /// Merges this [`TraitImpl`] with another.
    ///
    /// Update `self` with whichever value is not [`TraitImpl::NotImplemented`].
    /// If `other` is [`TraitImpl::NotImplemented`], then `self` is not modified.
    /// An error is returned if neither value is [`TraitImpl::NotImplemented`].
    pub fn merge(&mut self, other: TraitImpl) -> Result<(), syn::Error> {
        match (&self, other) {
            (TraitImpl::NotImplemented, value) => {
                *self = value;
                Ok(())
            }
            (_, TraitImpl::NotImplemented) => Ok(()),
            (_, TraitImpl::Implemented(span) | TraitImpl::Custom(_, span)) => {
                Err(syn::Error::new(span, CONFLICTING_TYPE_DATA_MESSAGE))
            }
        }
    }
}

/// A collection of attributes used for deriving `FromReflect`.
#[derive(Clone, Default)]
pub(crate) struct FromReflectAttrs {
    auto_derive: Option<LitBool>,
}

impl FromReflectAttrs {
    /// Returns true if `FromReflect` should be automatically derived as part of the `Reflect` derive.
    pub fn should_auto_derive(&self) -> bool {
        self.auto_derive.as_ref().is_none_or(LitBool::value)
    }
}

/// A collection of attributes used for deriving `TypePath` via the `Reflect` derive.
///
/// Note that this differs from the attributes used by the `TypePath` derive itself,
/// which look like `[type_path = "my_crate::foo"]`.
/// The attributes used by reflection take the form `#[reflect(type_path = false)]`.
///
/// These attributes should only be used for `TypePath` configuration specific to
/// deriving `Reflect`.
#[derive(Clone, Default)]
pub(crate) struct TypePathAttrs {
    auto_derive: Option<LitBool>,
}

impl TypePathAttrs {
    /// Returns true if `TypePath` should be automatically derived as part of the `Reflect` derive.
    pub fn should_auto_derive(&self) -> bool {
        self.auto_derive.as_ref().is_none_or(LitBool::value)
    }
}

/// Extract a boolean value from an expression.
///
/// The mapper exists so that the caller can conditionally choose to use the given
/// value or supply their own.
fn extract_bool(
    value: &Expr,
    mut mapper: impl FnMut(&LitBool) -> LitBool,
) -> Result<LitBool, syn::Error> {
    match value {
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(lit),
            ..
        }) => Ok(mapper(lit)),
        _ => Err(syn::Error::new(value.span(), "Expected a boolean value")),
    }
}

/// Adds an identifier to a vector of identifiers if it is not already present.
///
/// Returns an error if the identifier already exists in the list.
fn add_unique_ident(idents: &mut Vec<Ident>, ident: Ident) -> Result<(), syn::Error> {
    let ident_name = ident.to_string();
    if idents.iter().any(|i| i == ident_name.as_str()) {
        return Err(syn::Error::new(ident.span(), CONFLICTING_TYPE_DATA_MESSAGE));
    }

    idents.push(ident);
    Ok(())
}

/// A collection of traits that have been registered for a reflected type.
///
/// This keeps track of a few traits that are utilized internally for reflection
/// (we'll call these traits _special traits_ within this context), but it
/// will also keep track of all registered traits. Traits are registered as part of the
/// `Reflect` derive macro using the helper attribute: `#[reflect(...)]`.
///
/// The list of special traits are as follows:
/// * `Debug`
/// * `Hash`
/// * `PartialEq`
///
/// When registering a trait, there are a few things to keep in mind:
/// * Traits must have a valid `Reflect{}` struct in scope. For example, `Default`
///   needs `obel_reflect::prelude::ReflectDefault` in scope.
/// * Traits must be single path identifiers. This means you _must_ use `Default`
///   instead of `std::default::Default` (otherwise it will try to register `Reflectstd`!)
/// * A custom function may be supplied in place of an actual implementation
///   for the special traits (but still follows the same single-path identifier
///   rules as normal).
///
/// # Example
///
/// Registering the `Default` implementation:
///
/// ```ignore (obel_reflect is not accessible from this crate)
/// // Import ReflectDefault so it's accessible by the derive macro
/// use obel_reflect::prelude::ReflectDefault;
///
/// #[derive(Reflect, Default)]
/// #[reflect(Default)]
/// struct Foo;
/// ```
///
/// Registering the `Hash` implementation:
///
/// ```ignore (obel_reflect is not accessible from this crate)
/// // `Hash` is a "special trait" and does not need (nor have) a ReflectHash struct
///
/// #[derive(Reflect, Hash)]
/// #[reflect(Hash)]
/// struct Foo;
/// ```
///
/// Registering the `Hash` implementation using a custom function:
///
/// ```ignore (obel_reflect is not accessible from this crate)
/// // This function acts as our `Hash` implementation and
/// // corresponds to the `Reflect::reflect_hash` method.
/// fn get_hash(foo: &Foo) -> Option<u64> {
///   Some(123)
/// }
///
/// #[derive(Reflect)]
/// // Register the custom `Hash` function
/// #[reflect(Hash(get_hash))]
/// struct Foo;
/// ```
///
/// > __Note:__ Registering a custom function only works for special traits.
#[derive(Default, Clone)]
pub(crate) struct ContainerAttributes {
    debug: TraitImpl,
    hash: TraitImpl,
    partial_eq: TraitImpl,
    from_reflect_attrs: FromReflectAttrs,
    type_path_attrs: TypePathAttrs,
    custom_where: Option<WhereClause>,
    no_field_bounds: bool,
    custom_attributes: CustomAttributes,
    is_opaque: bool,
    idents: Vec<Ident>,
}

impl ContainerAttributes {
    /// Parse all field attributes marked "reflect" (such as `#[reflect(ignore)]`).
    pub fn parse_attributes(
        attrs: &[syn::Attribute],
        trait_: ReflectTraitToImpl,
    ) -> syn::Result<Self> {
        let mut args = ContainerAttributes::default();

        attrs
            .iter()
            .filter_map(|attr| {
                if !attr.path().is_ident(crate::REFLECT_ATTRIBUTE_NAME) {
                    // Not a reflect attribute -> skip
                    return None;
                }

                let syn::Meta::List(meta) = &attr.meta else {
                    return Some(syn::Error::new_spanned(attr, "expected meta list"));
                };

                // Parse all attributes inside the list, collecting any errors
                meta.parse_args_with(terminated_parser(Token![,], |stream| {
                    args.parse_container_attribute(stream, trait_)
                }))
                .err()
            })
            .reduce(|mut acc, err| {
                acc.combine(err);
                acc
            })
            .map_or(Ok(args), Err)
    }

    /// Parse the contents of a `#[reflect(...)]` attribute into a [`ContainerAttributes`] instance.
    ///
    /// # Example
    /// - `#[reflect(Hash, Debug(custom_debug), MyTrait)]`
    /// - `#[reflect(no_field_bounds)]`
    pub fn parse_meta_list(
        &mut self,
        meta: &MetaList,
        trait_: ReflectTraitToImpl,
    ) -> syn::Result<()> {
        meta.parse_args_with(|stream: ParseStream| self.parse_terminated(stream, trait_))
    }

    /// Parse a comma-separated list of container attributes.
    ///
    /// # Example
    /// - `Hash, Debug(custom_debug), MyTrait`
    pub fn parse_terminated(
        &mut self,
        input: ParseStream,
        trait_: ReflectTraitToImpl,
    ) -> syn::Result<()> {
        terminated_parser(Token![,], |stream| self.parse_container_attribute(stream, trait_))(
            input,
        )?;

        Ok(())
    }

    /// Parse a single container attribute.
    fn parse_container_attribute(
        &mut self,
        input: ParseStream,
        trait_: ReflectTraitToImpl,
    ) -> syn::Result<()> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![@]) {
            self.custom_attributes.parse_custom_attribute(input)
        } else if lookahead.peek(Token![where]) {
            self.parse_custom_where(input)
        } else if lookahead.peek(kw::from_reflect) {
            self.parse_from_reflect(input, trait_)
        } else if lookahead.peek(kw::type_path) {
            self.parse_type_path(input, trait_)
        } else if lookahead.peek(kw::opaque) {
            self.parse_opaque(input)
        } else if lookahead.peek(kw::no_field_bounds) {
            self.parse_no_field_bounds(input)
        } else if lookahead.peek(kw::Debug) {
            self.parse_debug(input)
        } else if lookahead.peek(kw::PartialEq) {
            self.parse_partial_eq(input)
        } else if lookahead.peek(kw::Hash) {
            self.parse_hash(input)
        } else if lookahead.peek(Ident::peek_any) {
            self.parse_ident(input)
        } else {
            Err(lookahead.error())
        }
    }

    /// Parse an ident (for registration).
    ///
    /// Examples:
    /// - `#[reflect(MyTrait)]` (registers `ReflectMyTrait`)
    fn parse_ident(&mut self, input: ParseStream) -> syn::Result<()> {
        let ident = input.parse::<Ident>()?;

        if input.peek(token::Paren) {
            return Err(syn::Error::new(
                ident.span(),
                format!(
                    "only [{DEBUG_ATTR:?}, {PARTIAL_EQ_ATTR:?}, {HASH_ATTR:?}] may specify custom functions",
                ),
            ));
        }

        let ident_name = ident.to_string();

        // Create the reflect ident
        let mut reflect_ident = crate::ident::get_reflect_ident(&ident_name);
        // We set the span to the old ident so any compile errors point to that ident instead
        reflect_ident.set_span(ident.span());

        add_unique_ident(&mut self.idents, reflect_ident)?;

        Ok(())
    }

    /// Parse special `Debug` registration.
    ///
    /// Examples:
    /// - `#[reflect(Debug)]`
    /// - `#[reflect(Debug(custom_debug_fn))]`
    fn parse_debug(&mut self, input: ParseStream) -> syn::Result<()> {
        let ident = input.parse::<kw::Debug>()?;

        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let path = content.parse::<Path>()?;
            self.debug.merge(TraitImpl::Custom(path, ident.span))?;
        } else {
            self.debug = TraitImpl::Implemented(ident.span);
        }

        Ok(())
    }

    /// Parse special `PartialEq` registration.
    ///
    /// Examples:
    /// - `#[reflect(PartialEq)]`
    /// - `#[reflect(PartialEq(custom_partial_eq_fn))]`
    fn parse_partial_eq(&mut self, input: ParseStream) -> syn::Result<()> {
        let ident = input.parse::<kw::PartialEq>()?;

        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let path = content.parse::<Path>()?;
            self.partial_eq.merge(TraitImpl::Custom(path, ident.span))?;
        } else {
            self.partial_eq = TraitImpl::Implemented(ident.span);
        }

        Ok(())
    }

    /// Parse special `Hash` registration.
    ///
    /// Examples:
    /// - `#[reflect(Hash)]`
    /// - `#[reflect(Hash(custom_hash_fn))]`
    fn parse_hash(&mut self, input: ParseStream) -> syn::Result<()> {
        let ident = input.parse::<kw::Hash>()?;

        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let path = content.parse::<Path>()?;
            self.hash.merge(TraitImpl::Custom(path, ident.span))?;
        } else {
            self.hash = TraitImpl::Implemented(ident.span);
        }

        Ok(())
    }

    /// Parse `opaque` attribute.
    ///
    /// Examples:
    /// - `#[reflect(opaque)]`
    fn parse_opaque(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<kw::opaque>()?;
        self.is_opaque = true;
        Ok(())
    }

    /// Parse `no_field_bounds` attribute.
    ///
    /// Examples:
    /// - `#[reflect(no_field_bounds)]`
    fn parse_no_field_bounds(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<kw::no_field_bounds>()?;
        self.no_field_bounds = true;
        Ok(())
    }

    /// Parse `where` attribute.
    ///
    /// Examples:
    /// - `#[reflect(where T: Debug)]`
    fn parse_custom_where(&mut self, input: ParseStream) -> syn::Result<()> {
        self.custom_where = Some(input.parse()?);
        Ok(())
    }

    /// Parse `from_reflect` attribute.
    ///
    /// Examples:
    /// - `#[reflect(from_reflect = false)]`
    fn parse_from_reflect(
        &mut self,
        input: ParseStream,
        trait_: ReflectTraitToImpl,
    ) -> syn::Result<()> {
        let pair = input.parse::<MetaNameValue>()?;
        let extracted_bool = extract_bool(&pair.value, |lit| {
            // Override `lit` if this is a `FromReflect` derive.
            // This typically means a user is opting out of the default implementation
            // from the `Reflect` derive and using the `FromReflect` derive directly instead.
            (trait_ == ReflectTraitToImpl::FromReflect)
                .then(|| LitBool::new(true, Span::call_site()))
                .unwrap_or_else(|| lit.clone())
        })?;

        if let Some(existing) = &self.from_reflect_attrs.auto_derive {
            if existing.value() != extracted_bool.value() {
                return Err(syn::Error::new(
                    extracted_bool.span(),
                    format!("`{FROM_REFLECT_ATTR}` already set to {}", existing.value()),
                ));
            }
        } else {
            self.from_reflect_attrs.auto_derive = Some(extracted_bool);
        }

        Ok(())
    }

    /// Parse `type_path` attribute.
    ///
    /// Examples:
    /// - `#[reflect(type_path = false)]`
    fn parse_type_path(
        &mut self,
        input: ParseStream,
        trait_: ReflectTraitToImpl,
    ) -> syn::Result<()> {
        let pair = input.parse::<MetaNameValue>()?;
        let extracted_bool = extract_bool(&pair.value, |lit| {
            // Override `lit` if this is a `FromReflect` derive.
            // This typically means a user is opting out of the default implementation
            // from the `Reflect` derive and using the `FromReflect` derive directly instead.
            (trait_ == ReflectTraitToImpl::TypePath)
                .then(|| LitBool::new(true, Span::call_site()))
                .unwrap_or_else(|| lit.clone())
        })?;

        if let Some(existing) = &self.type_path_attrs.auto_derive {
            if existing.value() != extracted_bool.value() {
                return Err(syn::Error::new(
                    extracted_bool.span(),
                    format!("`{TYPE_PATH_ATTR}` already set to {}", existing.value()),
                ));
            }
        } else {
            self.type_path_attrs.auto_derive = Some(extracted_bool);
        }

        Ok(())
    }

    /// Returns true if the given reflected trait name (i.e. `ReflectDefault` for `Default`)
    /// is registered for this type.
    pub fn contains(&self, name: &str) -> bool {
        self.idents.iter().any(|ident| ident == name)
    }

    /// The list of reflected traits by their reflected ident (i.e. `ReflectDefault` for `Default`).
    pub fn idents(&self) -> &[Ident] {
        &self.idents
    }

    /// The `FromReflect` configuration found within `#[reflect(...)]` attributes on this type.
    #[expect(
        clippy::wrong_self_convention,
        reason = "Method returns `FromReflectAttrs`, does not actually convert data."
    )]
    pub fn from_reflect_attrs(&self) -> &FromReflectAttrs {
        &self.from_reflect_attrs
    }

    /// The `TypePath` configuration found within `#[reflect(...)]` attributes on this type.
    pub fn type_path_attrs(&self) -> &TypePathAttrs {
        &self.type_path_attrs
    }

    pub fn custom_attributes(&self) -> &CustomAttributes {
        &self.custom_attributes
    }

    /// The custom where configuration found within `#[reflect(...)]` attributes on this type.
    pub fn custom_where(&self) -> Option<&WhereClause> {
        self.custom_where.as_ref()
    }

    /// Returns true if the `no_field_bounds` attribute was found on this type.
    pub fn no_field_bounds(&self) -> bool {
        self.no_field_bounds
    }

    /// Returns true if the `opaque` attribute was found on this type.
    pub fn is_opaque(&self) -> bool {
        self.is_opaque
    }

    /// Returns the implementation of `PartialReflect::reflect_hash` as a `TokenStream`.
    ///
    /// If `Hash` was not registered, returns `None`.
    pub fn get_hash_impl(&self, obel_reflect_path: &Path) -> Option<proc_macro2::TokenStream> {
        match &self.hash {
            &TraitImpl::Implemented(span) => Some(quote_spanned! {span=>
                fn reflect_hash(&self) -> #FQOption<u64> {
                    use ::core::hash::{Hash, Hasher};
                    let mut hasher = #obel_reflect_path::utility::reflect_hasher();
                    Hash::hash(&#FQAny::type_id(self), &mut hasher);
                    Hash::hash(self, &mut hasher);
                    #FQOption::Some(Hasher::finish(&hasher))
                }
            }),
            &TraitImpl::Custom(ref impl_fn, span) => Some(quote_spanned! {span=>
                fn reflect_hash(&self) -> #FQOption<u64> {
                    #FQOption::Some(#impl_fn(self))
                }
            }),
            TraitImpl::NotImplemented => None,
        }
    }

    /// Returns the implementation of `PartialReflect::reflect_partial_eq` as a `TokenStream`.
    ///
    /// If `PartialEq` was not registered, returns `None`.
    pub fn get_partial_eq_impl(
        &self,
        obel_reflect_path: &Path,
    ) -> Option<proc_macro2::TokenStream> {
        match &self.partial_eq {
            &TraitImpl::Implemented(span) => Some(quote_spanned! {span=>
                fn reflect_partial_eq(&self, value: &dyn #obel_reflect_path::PartialReflect) -> #FQOption<bool> {
                    let value = <dyn #obel_reflect_path::PartialReflect>::try_downcast_ref::<Self>(value);
                    if let #FQOption::Some(value) = value {
                        #FQOption::Some(::core::cmp::PartialEq::eq(self, value))
                    } else {
                        #FQOption::Some(false)
                    }
                }
            }),
            &TraitImpl::Custom(ref impl_fn, span) => Some(quote_spanned! {span=>
                fn reflect_partial_eq(&self, value: &dyn #obel_reflect_path::PartialReflect) -> #FQOption<bool> {
                    #FQOption::Some(#impl_fn(self, value))
                }
            }),
            TraitImpl::NotImplemented => None,
        }
    }

    /// Returns the implementation of `PartialReflect::debug` as a `TokenStream`.
    ///
    /// If `Debug` was not registered, returns `None`.
    pub fn get_debug_impl(&self) -> Option<proc_macro2::TokenStream> {
        match &self.debug {
            &TraitImpl::Implemented(span) => Some(quote_spanned! {span=>
                fn debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Debug::fmt(self, f)
                }
            }),
            &TraitImpl::Custom(ref impl_fn, span) => Some(quote_spanned! {span=>
                fn debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #impl_fn(self, f)
                }
            }),
            TraitImpl::NotImplemented => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;
    use syn::{Attribute, parse_quote};

    // Helper function to create a `#[reflect(...)]` attribute
    fn create_reflect_attribute(tokens: proc_macro2::TokenStream) -> Attribute {
        parse_quote!(#[reflect(#tokens)])
    }

    // Helper function to create a `ContainerAttributes` instance with a specific `TraitImpl`
    fn create_container_attributes_with_trait_impl(
        debug: TraitImpl,
        partial_eq: TraitImpl,
        hash: TraitImpl,
    ) -> ContainerAttributes {
        ContainerAttributes {
            debug,
            partial_eq,
            hash,
            ..Default::default()
        }
    }

    #[test]
    fn test_trait_impl_merge() {
        let mut trait_impl = TraitImpl::NotImplemented;
        let other = TraitImpl::Implemented(Span::call_site());

        assert!(trait_impl.merge(other).is_ok());
        assert!(matches!(trait_impl, TraitImpl::Implemented(_)));

        let other_custom = TraitImpl::Custom(parse_quote!(custom_fn), Span::call_site());
        assert!(trait_impl.merge(other_custom).is_err());
    }

    #[test]
    fn test_from_reflect_attrs_should_auto_derive() {
        let attrs = FromReflectAttrs {
            auto_derive: None,
        };
        assert!(attrs.should_auto_derive());

        let attrs = FromReflectAttrs {
            auto_derive: Some(LitBool::new(true, Span::call_site())),
        };
        assert!(attrs.should_auto_derive());

        let attrs = FromReflectAttrs {
            auto_derive: Some(LitBool::new(false, Span::call_site())),
        };
        assert!(!attrs.should_auto_derive());
    }

    #[test]
    fn test_type_path_attrs_should_auto_derive() {
        let attrs = TypePathAttrs {
            auto_derive: None,
        };
        assert!(attrs.should_auto_derive());

        let attrs = TypePathAttrs {
            auto_derive: Some(LitBool::new(true, Span::call_site())),
        };
        assert!(attrs.should_auto_derive());

        let attrs = TypePathAttrs {
            auto_derive: Some(LitBool::new(false, Span::call_site())),
        };
        assert!(!attrs.should_auto_derive());
    }

    #[test]
    fn test_extract_bool() {
        let expr = parse_quote!(true);
        let result = extract_bool(&expr, |lit| lit.clone());
        assert!(result.is_ok());
        assert!(result.unwrap().value());

        let expr = parse_quote!(false);
        let result = extract_bool(&expr, |lit| lit.clone());
        assert!(result.is_ok());
        assert!(!result.unwrap().value());

        let expr = parse_quote!("not a bool");
        let result = extract_bool(&expr, |lit| lit.clone());
        assert!(result.is_err());
    }

    #[test]
    fn test_add_unique_ident() {
        let mut idents = vec![];
        let ident = Ident::new("Test", Span::call_site());

        assert!(add_unique_ident(&mut idents, ident.clone()).is_ok());
        assert_eq!(idents.len(), 1);

        let result = add_unique_ident(&mut idents, ident);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_attributes_simple_trait() {
        // Test parsing a simple trait like `#[reflect(MyTrait)]`
        let attr = create_reflect_attribute(quote!(MyTrait));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that the trait was registered
        assert!(container_attrs.contains("ReflectMyTrait"));
    }

    #[test]
    fn test_parse_attributes_special_traits() {
        // Test parsing special traits like `Debug`, `PartialEq`, and `Hash`
        let attr = create_reflect_attribute(quote!(Debug, PartialEq, Hash));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that the special traits were registered
        assert!(matches!(container_attrs.debug, TraitImpl::Implemented(_)));
        assert!(matches!(container_attrs.partial_eq, TraitImpl::Implemented(_)));
        assert!(matches!(container_attrs.hash, TraitImpl::Implemented(_)));
    }

    #[test]
    fn test_parse_attributes_custom_functions() {
        // Test parsing custom functions for special traits
        let attr = create_reflect_attribute(quote!(
            Debug(custom_debug),
            PartialEq(custom_partial_eq),
            Hash(custom_hash)
        ));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that the custom functions were registered
        assert!(matches!(container_attrs.debug, TraitImpl::Custom(_, _)));
        assert!(matches!(container_attrs.partial_eq, TraitImpl::Custom(_, _)));
        assert!(matches!(container_attrs.hash, TraitImpl::Custom(_, _)));
    }

    #[test]
    fn test_parse_attributes_from_reflect() {
        // Test parsing the `from_reflect` attribute
        let attr = create_reflect_attribute(quote!(from_reflect = true));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that `from_reflect` was set to `true`
        assert!(container_attrs.from_reflect_attrs().should_auto_derive());

        // Test with `false`
        let attr = create_reflect_attribute(quote!(from_reflect = false));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that `from_reflect` was set to `false`
        assert!(!container_attrs.from_reflect_attrs().should_auto_derive());
    }

    #[test]
    fn test_parse_attributes_type_path() {
        // Test parsing the `type_path` attribute
        let attr = create_reflect_attribute(quote!(type_path = true));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that `type_path` was set to `true`
        assert!(container_attrs.type_path_attrs().should_auto_derive());

        // Test with `false`
        let attr = create_reflect_attribute(quote!(type_path = false));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that `type_path` was set to `false`
        assert!(!container_attrs.type_path_attrs().should_auto_derive());
    }

    #[test]
    fn test_parse_attributes_opaque() {
        // Test parsing the `opaque` attribute
        let attr = create_reflect_attribute(quote!(opaque));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that `opaque` was set
        assert!(container_attrs.is_opaque());
    }

    #[test]
    fn test_parse_attributes_no_field_bounds() {
        // Test parsing the `no_field_bounds` attribute
        let attr = create_reflect_attribute(quote!(no_field_bounds));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that `no_field_bounds` was set
        assert!(container_attrs.no_field_bounds());
    }

    #[test]
    fn test_parse_attributes_custom_where() {
        // Test parsing a custom `where` clause
        let attr = create_reflect_attribute(quote!(where T: Debug));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that the custom `where` clause was set
        assert!(container_attrs.custom_where().is_some());
    }

    #[test]
    fn test_parse_attributes_multiple_attributes() {
        // Test parsing multiple attributes in a single `#[reflect(...)]`
        let attr =
            create_reflect_attribute(quote!(Debug, PartialEq, Hash, no_field_bounds, opaque));
        let container_attrs =
            ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect).unwrap();

        // Verify that all attributes were parsed correctly
        assert!(matches!(container_attrs.debug, TraitImpl::Implemented(_)));
        assert!(matches!(container_attrs.partial_eq, TraitImpl::Implemented(_)));
        assert!(matches!(container_attrs.hash, TraitImpl::Implemented(_)));
        assert!(container_attrs.no_field_bounds());
        assert!(container_attrs.is_opaque());
    }

    #[test]
    fn test_parse_attributes_conflicting_traits() {
        // Test parsing conflicting trait registrations
        let attr = create_reflect_attribute(quote!(Debug, Debug(custom_debug)));
        let result = ContainerAttributes::parse_attributes(&[attr], ReflectTraitToImpl::Reflect);

        // Verify that an error was returned
        assert!(result.is_err());
    }

    #[test]
    fn test_get_debug_impl_implemented() {
        // Test `get_debug_impl` for an implemented trait
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::Implemented(Span::call_site()),
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
        );

        let debug_impl = container_attrs.get_debug_impl().unwrap();
        let expected = quote! {
            fn debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self, f)
            }
        };

        assert_eq!(debug_impl.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_debug_impl_custom() {
        // Test `get_debug_impl` for a custom implementation
        let custom_fn: Path = parse_quote!(custom_debug_fn);
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::Custom(custom_fn.clone(), Span::call_site()),
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
        );

        let debug_impl = container_attrs.get_debug_impl().unwrap();
        let expected = quote! {
            fn debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #custom_fn(self, f)
            }
        };

        assert_eq!(debug_impl.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_debug_impl_not_implemented() {
        // Test `get_debug_impl` when the trait is not implemented
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
        );

        let debug_impl = container_attrs.get_debug_impl();
        assert!(debug_impl.is_none());
    }

    #[test]
    fn test_get_partial_eq_impl_implemented() {
        // Test `get_partial_eq_impl` for an implemented trait
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::Implemented(Span::call_site()),
            TraitImpl::NotImplemented,
        );

        let obel_reflect_path: Path = parse_quote!(obel_reflect);
        let partial_eq_impl = container_attrs.get_partial_eq_impl(&obel_reflect_path).unwrap();

        let expected = quote! {
            fn reflect_partial_eq(&self, value: &dyn obel_reflect::PartialReflect) -> ::core::option::Option<bool> {
                let value = <dyn obel_reflect::PartialReflect>::try_downcast_ref::<Self>(value);
                if let ::core::option::Option::Some(value) = value {
                    ::core::option::Option::Some(::core::cmp::PartialEq::eq(self, value))
                } else {
                    ::core::option::Option::Some(false)
                }
            }
        };

        assert_eq!(partial_eq_impl.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_partial_eq_impl_custom() {
        // Test `get_partial_eq_impl` for a custom implementation
        let custom_fn: Path = parse_quote!(custom_partial_eq_fn);
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::Custom(custom_fn.clone(), Span::call_site()),
            TraitImpl::NotImplemented,
        );

        let obel_reflect_path: Path = parse_quote!(obel_reflect);
        let partial_eq_impl = container_attrs.get_partial_eq_impl(&obel_reflect_path).unwrap();

        let expected = quote! {
            fn reflect_partial_eq(&self, value: &dyn obel_reflect::PartialReflect) -> ::core::option::Option<bool> {
                ::core::option::Option::Some(#custom_fn(self, value))
            }
        };

        assert_eq!(partial_eq_impl.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_partial_eq_impl_not_implemented() {
        // Test `get_partial_eq_impl` when the trait is not implemented
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
        );

        let obel_reflect_path: Path = parse_quote!(obel_reflect);
        let partial_eq_impl = container_attrs.get_partial_eq_impl(&obel_reflect_path);
        assert!(partial_eq_impl.is_none());
    }

    #[test]
    fn test_get_hash_impl_implemented() {
        // Test `get_hash_impl` for an implemented trait
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
            TraitImpl::Implemented(Span::call_site()),
        );

        let obel_reflect_path: Path = parse_quote!(obel_reflect);
        let hash_impl = container_attrs.get_hash_impl(&obel_reflect_path).unwrap();

        let expected = quote! {
            fn reflect_hash(&self) -> ::core::option::Option<u64> {
                use ::core::hash::{Hash, Hasher};
                let mut hasher = obel_reflect::utility::reflect_hasher();
                Hash::hash(&::core::any::Any::type_id(self), &mut hasher);
                Hash::hash(self, &mut hasher);
                ::core::option::Option::Some(Hasher::finish(&hasher))
            }
        };

        assert_eq!(hash_impl.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_hash_impl_custom() {
        // Test `get_hash_impl` for a custom implementation
        let custom_fn: Path = parse_quote!(custom_hash_fn);
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
            TraitImpl::Custom(custom_fn.clone(), Span::call_site()),
        );

        let obel_reflect_path: Path = parse_quote!(obel_reflect);
        let hash_impl = container_attrs.get_hash_impl(&obel_reflect_path).unwrap();

        let expected = quote! {
            fn reflect_hash(&self) -> ::core::option::Option<u64> {
                ::core::option::Option::Some(#custom_fn(self))
            }
        };

        assert_eq!(hash_impl.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_hash_impl_not_implemented() {
        // Test `get_hash_impl` when the trait is not implemented
        let container_attrs = create_container_attributes_with_trait_impl(
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
            TraitImpl::NotImplemented,
        );

        let obel_reflect_path: Path = parse_quote!(obel_reflect);
        let hash_impl = container_attrs.get_hash_impl(&obel_reflect_path);
        assert!(hash_impl.is_none());
    }
}
