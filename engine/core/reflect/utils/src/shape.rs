use proc_macro2::Span;
use syn::{punctuated::Punctuated, token::Comma, Data, DataStruct, Error, Field, Fields};

/// Get the fields of a data structure if that structure is a struct with named fields;
/// otherwise, return a compile error that points to the site of the macro invocation.
pub fn get_struct_fields(data: &Data) -> syn::Result<&Punctuated<Field, Comma>> {
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => Ok(&fields.named),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => Ok(&fields.unnamed),
        _ => Err(Error::new(
            // This deliberately points to the call site rather than the structure
            // body; marking the entire body as the source of the error makes it
            // impossible to figure out which `derive` has a problem.
            Span::call_site(),
            "Only structs are supported",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::{parse_quote, DeriveInput};

    #[test]
    fn test_struct() {
        let input: DeriveInput = parse_quote! {
            struct Test {
                field1: i32,
                field2: String,
            }
        };
        let data = input.data;
        let result = get_struct_fields(&data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.to_token_stream().to_string(), "field1 : i32 , field2 : String ,");

        let input: DeriveInput = parse_quote! {
            struct Test(i32, String);
        };
        let data = input.data;
        let result = get_struct_fields(&data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.to_token_stream().to_string(), "i32 , String");
    }

    #[test]
    fn test_returns_error() {
        let input: DeriveInput = parse_quote! {
            union Test {
                f1: u32,
                f2: f32,
            }
        };
        let data = input.data;
        assert!(get_struct_fields(&data).is_err());
        let input: DeriveInput = parse_quote! {
            enum Test {
                Variant1,
                Variant2(i32),
            }
        };
        let data = input.data;
        assert!(get_struct_fields(&data).is_err());
    }
}
