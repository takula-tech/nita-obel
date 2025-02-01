#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std]

pub use obel_api::*;

// Wasm does not support dynamic linking.
#[cfg(all(feature = "dynamic_linking", not(target_family = "wasm")))]
#[expect(
    unused_imports,
    clippy::single_component_path_imports,
    reason = "This causes obel to be compiled as a dylib when using dynamic linking, and as such cannot be removed or changed without affecting dynamic linking."
)]
use obel_dylib;
