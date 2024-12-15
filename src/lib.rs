// Suppresses warnings about importing whole modules or crates (like use std;).
#![allow(clippy::single_component_path_imports)]
// Adds helpful annotations to your documentation, but only when building docs on docs.rs.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! doc
pub use obel_root::*;

// Wasm does not support dynamic linking.
#[cfg(all(feature = "dynamic_linking", not(target_family = "wasm")))]
#[allow(unused_imports)]
use obel_dylib;
