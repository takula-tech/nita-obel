use proc_macro2::TokenStream;
use std::{env, format, path::PathBuf, sync::LazyLock};
use toml_edit::{DocumentMut, Item};

/// The path to the `Cargo.toml` file for the Obel project.
pub struct ObelManifest {
    manifest: DocumentMut,
}

const OBEL: &str = "obel";
// const OBEL_API: &str = "obel_api";

impl ObelManifest {
    /// Returns a global shared instance of the [`ObelManifest`] struct.
    pub fn shared() -> &'static LazyLock<Self> {
        static LAZY_SELF: LazyLock<ObelManifest> = LazyLock::new(|| ObelManifest {
            manifest: env::var_os("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .map(|mut path| {
                    path.push("Cargo.toml");
                    if !path.exists() {
                        panic!("No Cargo manifest found for crate. Expected: {}", path.display());
                    }
                    let manifest = std::fs::read_to_string(path.clone()).unwrap_or_else(|_| {
                        panic!("Unable to read cargo manifest: {}", path.display())
                    });
                    manifest.parse::<DocumentMut>().unwrap_or_else(|_| {
                        panic!("Failed to parse cargo manifest: {}", path.display())
                    })
                })
                .expect("CARGO_MANIFEST_DIR is not defined."),
        });
        &LAZY_SELF
    }

    /// Attempt to retrieve the [path](syn::Path) of a particular package in
    /// the [manifest](ObelManifest) by [name](str).
    pub fn maybe_get_path(&self, name: &str) -> Option<syn::Path> {
        fn alias_name(dep: &Item) -> Option<&str> {
            if dep.as_str().is_some() {
                None
            } else {
                dep.get("package").map(|name| name.as_str().unwrap())
            }
        }
        let find_in_deps = |deps: &Item| -> Option<syn::Path> {
            if let Some(dep) = deps.get(name) {
                let path = Self::parse_str::<syn::Path>(alias_name(dep).unwrap_or(name));
                Some(path)
            } else {
                None
            }
        };
        let deps = self.manifest.get("dependencies");
        let deps_dev = self.manifest.get("dev-dependencies");
        deps.and_then(find_in_deps).or_else(|| deps_dev.and_then(find_in_deps))
    }

    /// Returns the path for the crate with the given name.
    /// the crate where the macro is used can have dependency of obel or obel_api
    /// or even the obel dep is renamed with package = 'xxx'. so must cal the crate
    /// path on the fly.
    pub fn get_path(&self, name: &str) -> syn::Path {
        self.maybe_get_path(name).unwrap_or_else(|| Self::parse_str(name))
    }

    /// Attempt to parse the provided [path](str) as a [syntax tree node](syn::parse::Parse)
    pub fn try_parse_str<T: syn::parse::Parse>(path: &str) -> Option<T> {
        syn::parse2(path.parse::<TokenStream>().ok()?).ok()
    }

    /// Attempt to parse provided [path](str) as a [syntax tree node](syn::parse::Parse).
    ///
    /// # Panics
    ///
    /// Will panic if the path is not able to be parsed. For a non-panicking option, see [`try_parse_str`]
    ///
    /// [`try_parse_str`]: Self::try_parse_str
    pub fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
        Self::try_parse_str(path).unwrap()
    }

    /// Attempt to get a subcrate [path](syn::Path) under Obel by [name](str)
    pub fn get_subcrate(&self, subcrate: &str) -> Option<syn::Path> {
        self.maybe_get_path(OBEL)
            .map(|obel_path| {
                let mut segments = obel_path.segments;
                segments.push(ObelManifest::parse_str(subcrate));
                syn::Path {
                    leading_colon: None,
                    segments,
                }
            })
            .or_else(|| self.maybe_get_path(&format!("obel_{subcrate}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_str() {
        let path = ObelManifest::parse_str::<syn::Path>("test::path");
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "test");
        assert_eq!(path.segments[1].ident.to_string(), "path");
    }

    #[test]
    fn test_try_parse_str() {
        assert!(ObelManifest::try_parse_str::<syn::Path>("test::path").is_some());
        assert!(ObelManifest::try_parse_str::<syn::Path>("invalid::path::").is_none());
    }

    #[test]
    fn test_maybe_get_path() {
        let manifest = ObelManifest {
            manifest: r#"
              [package]
              name = "test_crate"
              version = "0.1.0"
              [dependencies]
              obel = { version = "0.1.0" }
              test_dep = "1.0.0"
              renamed_dep = { package = "actual_name", version = "1.0.0" }
              [dev-dependencies]
              dev_dep = "1.0.0"
            "#
            .parse()
            .unwrap(),
        };
        // Test direct dependency
        let path = manifest.maybe_get_path("test_dep").unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "test_dep");

        // Test direct&renamed dependency
        let path = manifest.maybe_get_path("renamed_dep").unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "actual_name");

        // Test direct dev dependency
        let path = manifest.maybe_get_path("dev_dep").unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "dev_dep");

        // Test non-existent dependency
        assert!(manifest.maybe_get_path("nonexistent").is_none());
    }

    #[test]
    fn test_get_subcrate() {
        // Test subcrate with obel prefix
        let manifest = ObelManifest {
            manifest: r#"
            [package]
            name = "test_crate"
            version = "0.1.0"
            [dependencies]
            obel = { version = "0.1.0" }
            test_dep = "1.0.0"
            renamed_dep = { package = "actual_name", version = "1.0.0" }
            [dev-dependencies]
            dev_dep = "1.0.0"
            "#
            .parse()
            .unwrap(),
        };
        let path = manifest.get_subcrate("test").unwrap();
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0].ident.to_string(), "obel");
        assert_eq!(path.segments[1].ident.to_string(), "test");

        // Test subcrate with obel_test crate and test subcrate
        let manifest = ObelManifest {
            manifest: r#"
          [package]
          name = "test_crate"
          version = "0.1.0"
          [dependencies]
          obel_test = { version = "0.1.0" }
          "#
            .parse()
            .unwrap(),
        };
        let path = manifest.get_subcrate("test").unwrap();
        assert_eq!(path.segments.len(), 1);
        assert_eq!(path.segments[0].ident.to_string(), "obel_test");

        // Test subcrate with obel_test crate and obel_test subcrate
        let manifest = ObelManifest {
            manifest: r#"
              [package]
              name = "test_crate"
              version = "0.1.0"
              [dependencies]
              obel_test = { version = "0.1.0" }
              "#
            .parse()
            .unwrap(),
        };
        assert!(manifest.get_subcrate("obel_test").is_none());

        // Test non-existent subcrate
        let manifest = ObelManifest {
            manifest: r#"
          [package]
          name = "test_crate"
          version = "0.1.0"
          "#
            .parse()
            .unwrap(),
        };
        assert!(manifest.get_subcrate("nonexistent").is_none());
    }
}
