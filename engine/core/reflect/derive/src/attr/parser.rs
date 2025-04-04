use syn::{
    parse::{Parse, ParseStream, Peek},
    punctuated::Punctuated,
};

/// Returns a [`syn::parse::Parser`] which parses a stream of zero or more occurrences of `T`
/// separated by punctuation of type `P`, with optional trailing punctuation.
///
/// This is functionally the same as [`Punctuated::parse_terminated`],
/// but accepts a closure rather than a function pointer.
pub(crate) fn terminated_parser<T, P, F: FnMut(ParseStream) -> syn::Result<T>>(
    terminator: P,
    mut parser: F,
) -> impl FnOnce(ParseStream) -> syn::Result<Punctuated<T, P::Token>>
where
    P: Peek,
    P::Token: Parse,
{
    let _ = terminator;
    move |stream: ParseStream| {
        let mut punctuated = Punctuated::new();

        loop {
            if stream.is_empty() {
                break;
            }
            let value = parser(stream)?;
            punctuated.push_value(value);
            if stream.is_empty() {
                break;
            }
            let punct = stream.parse()?;
            punctuated.push_punct(punct);
        }

        Ok(punctuated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Token, parse::Parser};

    #[test]
    fn test_terminated_parser_empty_input() {
        let input = "";
        let parser =
            terminated_parser(Token![,], |stream: ParseStream| stream.parse::<syn::Ident>());
        let result = parser.parse_str(input);
        assert!(result.is_ok());
        let punctuated = result.unwrap();
        assert!(punctuated.is_empty());
    }

    #[test]
    fn test_terminated_parser_single_item() {
        let input = "foo";
        let parser =
            terminated_parser(Token![,], |stream: ParseStream| stream.parse::<syn::Ident>());
        let result = parser.parse_str(input);
        assert!(result.is_ok());
        let punctuated = result.unwrap();
        assert_eq!(punctuated.len(), 1);
        assert_eq!(punctuated.first().unwrap().to_string(), "foo");
    }

    #[test]
    fn test_terminated_parser_multiple_items() {
        let input = "foo, bar, baz";
        let parser =
            terminated_parser(Token![,], |stream: ParseStream| stream.parse::<syn::Ident>());
        let result = parser.parse_str(input);
        assert!(result.is_ok());
        let punctuated = result.unwrap();
        assert_eq!(punctuated.len(), 3);
        assert_eq!(punctuated[0].to_string(), "foo");
        assert_eq!(punctuated[1].to_string(), "bar");
        assert_eq!(punctuated[2].to_string(), "baz");
    }

    #[test]
    fn test_terminated_parser_trailing_punctuation() {
        let input = "foo, bar, baz,";
        let parser =
            terminated_parser(Token![,], |stream: ParseStream| stream.parse::<syn::Ident>());
        let result = parser.parse_str(input);
        assert!(result.is_ok());
        let punctuated = result.unwrap();
        assert_eq!(punctuated.len(), 3);
        assert_eq!(punctuated[0].to_string(), "foo");
        assert_eq!(punctuated[1].to_string(), "bar");
        assert_eq!(punctuated[2].to_string(), "baz");
    }

    #[test]
    fn test_terminated_parser_invalid_input() {
        let input = "foo, 123, baz";
        let parser =
            terminated_parser(Token![,], |stream: ParseStream| stream.parse::<syn::Ident>());
        let result = parser.parse_str(input);
        assert!(result.is_err());
        let _ = result.inspect_err(|e| {
            assert_eq!(e.to_string(), "expected identifier");
        });
    }
}
