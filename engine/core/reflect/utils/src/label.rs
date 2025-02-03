use obel_platform::{
    collections::HashSet, string::format, string::String, string::ToString, vec::Vec,
};
use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident};

/// Finds an identifier that will not conflict with the specified set of tokens.
///
/// If the identifier is present in `haystack`, extra characters will be added
/// to it until it no longer conflicts with anything.
///
/// Note that the returned identifier can still conflict in niche cases,
/// such as if an identifier in `haystack` is hidden behind an un-expanded macro.
pub fn ensure_no_collision(value: Ident, haystack: TokenStream) -> Ident {
    // Collect all the identifiers in `haystack` into a set.
    let idents = {
        // List of token streams that will be visited in future loop iterations.
        let mut unvisited = Vec::from([haystack]);
        // Identifiers we have found while searching tokens.
        let mut found = HashSet::<String>::default();
        while let Some(tokens) = unvisited.pop() {
            for t in tokens {
                match t {
                    // Collect any identifiers we encounter.
                    TokenTree::Ident(ident) => {
                        found.insert(ident.to_string());
                    }
                    // Queue up nested token streams to be visited in a future loop iteration.
                    TokenTree::Group(g) => unvisited.push(g.stream()),
                    TokenTree::Punct(_) | TokenTree::Literal(_) => {}
                }
            }
        }

        found
    };

    let span = value.span();

    // If there's a collision, add more characters to the identifier
    // until it doesn't collide with anything anymore.
    let mut value = value.to_string();
    while idents.contains(&value) {
        value.push('X');
    }

    Ident::new(&value, span)
}

/// Derive a label trait
///
/// # Args
///
/// - `input`: The [`syn::DeriveInput`] for struct that is deriving the label trait
/// - `trait_name`: Name of the label trait
/// - `trait_path`: The [path](`syn::Path`) to the label trait
/// - `dyn_eq_path`: The [path](`syn::Path`) to the `DynEq` trait
pub fn derive_label(
    input: syn::DeriveInput,
    trait_name: &str,
    trait_path: &syn::Path,
    dyn_eq_path: &syn::Path,
) -> TokenStream {
    if let syn::Data::Union(_) = &input.data {
        let message = format!("Cannot derive {trait_name} for unions.");
        return quote_spanned! {
            input.span() => compile_error!(#message);
        };
    }

    let ident = input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause.predicates.push(
        syn::parse2(quote! {
            Self: 'static + Send + Sync + Clone + Eq + ::core::fmt::Debug + ::core::hash::Hash
        })
        .unwrap(),
    );
    quote! {
        // To ensure alloc is available, but also prevent its name from clashing, we place the implementation inside an anonymous constant
        const _: () = {
            extern crate alloc;

            impl #impl_generics #trait_path for #ident #ty_generics #where_clause {
                fn dyn_clone(&self) -> alloc::boxed::Box<dyn #trait_path> {
                    alloc::boxed::Box::new(::core::clone::Clone::clone(self))
                }

                fn as_dyn_eq(&self) -> &dyn #dyn_eq_path {
                    self
                }

                fn dyn_hash(&self, mut state: &mut dyn ::core::hash::Hasher) {
                    let ty_id = ::core::any::TypeId::of::<Self>();
                    ::core::hash::Hash::hash(&ty_id, &mut state);
                    ::core::hash::Hash::hash(self, &mut state);
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::parse_quote;

    #[test]
    fn test_ensure_no_collision() {
        // Test case 1: No collision
        let ident = Ident::new("unique_name", Span::call_site());
        let haystack = quote! { existing_name another_name };
        let result = ensure_no_collision(ident, haystack);
        assert_eq!(result.to_string(), "unique_name");

        // Test case 2: Single collision
        let ident = Ident::new("test", Span::call_site());
        let haystack = quote! { test other_name };
        let result = ensure_no_collision(ident, haystack);
        assert_eq!(result.to_string(), "testX");

        // Test case 3: Multiple collisions requiring multiple X's
        let ident = Ident::new("test", Span::call_site());
        let haystack = quote! { test testX other_name };
        let result = ensure_no_collision(ident, haystack);
        assert_eq!(result.to_string(), "testXX");

        // Test case 4: Nested group tokens
        let ident = Ident::new("nested", Span::call_site());
        let haystack = quote! {
            outer {
                nested
                inner
            }
        };
        let result = ensure_no_collision(ident, haystack);
        assert_eq!(result.to_string(), "nestedX");
    }

    #[test]
    fn test_derive_label() {
        // Test case 1: Simple struct derivation
        let input: syn::DeriveInput = parse_quote! {
            struct TestStruct {
                field: i32
            }
        };
        let trait_name = "TestLabel";
        let trait_path: syn::Path = parse_quote!(TestLabel);
        let dyn_eq_path: syn::Path = parse_quote!(DynEq);

        let result = derive_label(input, trait_name, &trait_path, &dyn_eq_path);
        assert!(result.to_string() == "const _ : () = { extern crate alloc ; impl TestLabel for TestStruct where Self : 'static + Send + Sync + Clone + Eq + :: core :: fmt :: Debug + :: core :: hash :: Hash { fn dyn_clone (& self) -> alloc :: boxed :: Box < dyn TestLabel > { alloc :: boxed :: Box :: new (:: core :: clone :: Clone :: clone (self)) } fn as_dyn_eq (& self) -> & dyn DynEq { self } fn dyn_hash (& self , mut state : & mut dyn :: core :: hash :: Hasher) { let ty_id = :: core :: any :: TypeId :: of :: < Self > () ; :: core :: hash :: Hash :: hash (& ty_id , & mut state) ; :: core :: hash :: Hash :: hash (self , & mut state) ; } } } ;");

        // Test case 2: Union type (should return compile error)
        let union_input: syn::DeriveInput = parse_quote! {
            union TestUnion {
                field: i32
            }
        };
        let result = derive_label(union_input, trait_name, &trait_path, &dyn_eq_path);
        assert!(
            result.to_string() == "compile_error ! (\"Cannot derive TestLabel for unions.\") ;"
        );

        // Test case 3: Generic struct
        let generic_input: syn::DeriveInput = parse_quote! {
            struct GenericStruct<T: Clone> {
                field: T
            }
        };
        let result = derive_label(generic_input, trait_name, &trait_path, &dyn_eq_path);
        assert!(result.to_string() == "const _ : () = { extern crate alloc ; impl < T : Clone > TestLabel for GenericStruct < T > where Self : 'static + Send + Sync + Clone + Eq + :: core :: fmt :: Debug + :: core :: hash :: Hash { fn dyn_clone (& self) -> alloc :: boxed :: Box < dyn TestLabel > { alloc :: boxed :: Box :: new (:: core :: clone :: Clone :: clone (self)) } fn as_dyn_eq (& self) -> & dyn DynEq { self } fn dyn_hash (& self , mut state : & mut dyn :: core :: hash :: Hasher) { let ty_id = :: core :: any :: TypeId :: of :: < Self > () ; :: core :: hash :: Hash :: hash (& ty_id , & mut state) ; :: core :: hash :: Hash :: hash (self , & mut state) ; } } } ;");
    }
}
