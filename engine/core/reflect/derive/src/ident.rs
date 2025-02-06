use proc_macro2::{Ident, Span};
use syn::Member;

/// Returns the "reflected" ident for a given string.
///
/// # Example
///
/// ```
/// # use proc_macro2::Ident;
/// # // We can't import this method because of its visibility.
/// # fn get_reflect_ident(name: &str) -> Ident {
/// #     let reflected = format!("Reflect{name}");
/// #     Ident::new(&reflected, proc_macro2::Span::call_site())
/// # }
/// let reflected: Ident = get_reflect_ident("Hash");
/// assert_eq!("ReflectHash", reflected.to_string());
/// ```
pub(crate) fn get_reflect_ident(name: &str) -> Ident {
    let reflected = format!("Reflect{name}");
    Ident::new(&reflected, Span::call_site())
}

/// Returns a [`Member`] made of `ident` or `index` if `ident` is `None`.
///
/// Rust struct syntax allows for `Struct { foo: "string" }` with explicitly
/// named fields. It allows the `Struct { 0: "string" }` syntax when the struct
/// is declared as a tuple struct.
///
/// ```
/// struct Foo { field: &'static str }
/// struct Bar(&'static str);
/// let Foo { field } = Foo { field: "hi" };
/// let Bar { 0: field } = Bar { 0: "hello" };
/// let Bar(field) = Bar("hello"); // more common syntax
/// ```
///
/// This function helps field access in contexts where you are declaring either
/// a tuple struct or a struct with named fields. If you don't have a field name,
/// it means that you must access the field through an index.
pub(crate) fn ident_or_index(ident: Option<&Ident>, index: usize) -> Member {
    ident.map_or_else(|| Member::Unnamed(index.into()), |ident| Member::Named(ident.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_reflect_ident() {
        let cases = [
            ("Hash", "ReflectHash"),
            ("Debug", "ReflectDebug"),
            ("Custom", "ReflectCustom"),
            ("", "Reflect"),
        ];
        for (input, expected) in cases {
            let reflected = get_reflect_ident(input);
            assert_eq!(expected, reflected.to_string());
        }
    }

    #[test]
    fn test_ident_or_index_with_named_fields() {
        let field_name = Ident::new("field", Span::call_site());
        let member = ident_or_index(Some(&field_name), 0);
        assert!(matches!(member, Member::Named(_)));
        if let Member::Named(ident) = member {
            assert_eq!("field", ident.to_string());
        }
    }

    #[test]
    fn test_ident_or_index_with_unnamed_fields() {
        let member = ident_or_index(None, 0);
        assert!(matches!(member, Member::Unnamed(_)));
        if let Member::Unnamed(index) = member {
            assert_eq!(0, index.index);
        }
        let member = ident_or_index(None, 42);
        if let Member::Unnamed(index) = member {
            assert_eq!(42, index.index);
        }
    }
}
