//! Helper struct used to process an iterator of `Result<Vec<T>, syn::Error>`, combining errors into one along the way.
pub(crate) struct ResultSifter<T> {
    items: Vec<T>,
    errors: Option<syn::Error>,
}

impl<T> Default for ResultSifter<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            errors: None,
        }
    }
}

impl<T> ResultSifter<T> {
    /// Sift the given result, combining errors if necessary.
    pub fn sift(&mut self, result: Result<T, syn::Error>) {
        match result {
            Ok(data) => self.items.push(data),
            Err(err) => {
                if let Some(ref mut errors) = self.errors {
                    errors.combine(err);
                } else {
                    self.errors = Some(err);
                }
            }
        }
    }

    /// Associated method that provides a convenient implementation for [`Iterator::fold`].
    pub fn fold(mut sifter: Self, result: Result<T, syn::Error>) -> Self {
        sifter.sift(result);
        sifter
    }

    /// Complete the sifting process and return the final result.
    pub fn finish(self) -> Result<Vec<T>, syn::Error> {
        if let Some(errors) = self.errors {
            Err(errors)
        } else {
            Ok(self.items)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::Error;

    #[test]
    fn test_sift_success() {
        let mut sifter = ResultSifter::<i32>::default();
        sifter.sift(Ok(1));
        sifter.sift(Ok(2));
        let result = sifter.finish();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2]);
    }

    #[test]
    fn test_sift_single_error() {
        let mut sifter = ResultSifter::<i32>::default();
        sifter.sift(Err(Error::new(Span::call_site(), "test error")));
        let result = sifter.finish();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "test error");
    }

    #[test]
    fn test_sift_combined_errors() {
        let mut sifter = ResultSifter::<i32>::default();
        sifter.sift(Err(Error::new(Span::call_site(), "error 1")));
        sifter.sift(Err(Error::new(Span::call_site(), "error 2")));
        let result = sifter.finish();
        assert!(result.is_err());
        let err = result.unwrap_err().to_compile_error().to_string();
        assert!(err.contains("error 1"));
        assert!(err.contains("error 2"));
    }

    #[test]
    fn test_sift_mixed_results() {
        let mut sifter = ResultSifter::<i32>::default();
        sifter.sift(Ok(1));
        sifter.sift(Err(Error::new(Span::call_site(), "test error")));
        sifter.sift(Ok(2));
        let result = sifter.finish();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "test error");
    }

    #[test]
    fn test_fold() {
        let results = vec![Ok(1), Ok(2), Ok(3)];
        let sifter = results.into_iter().fold(ResultSifter::default(), ResultSifter::fold);
        let result = sifter.finish();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_fold_with_errors() {
        let results = vec![
            Ok(1),
            Err(Error::new(Span::call_site(), "fold error 1")),
            Ok(2),
            Err(Error::new(Span::call_site(), "fold error 2")),
        ];
        let sifter = results.into_iter().fold(ResultSifter::default(), ResultSifter::fold);
        let result = sifter.finish();
        assert!(result.is_err());
        let err = result.unwrap_err().to_compile_error().to_string();
        assert!(err.contains("fold error 1"));
        assert!(err.contains("fold error 2"));
    }
}
