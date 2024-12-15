#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! [![Obel Logo](https://nita-obel.takulatech.net/assets/bevy_logo_docs.svg)](https://nita-obel.takulatech.net/docs)
//!
//! Obel is an open-source modular game engine built in Rust, with a focus on developer productivity
//! and performance.
//!
//! Check out the [Obel website](https://nita-obel.takulatech.net) for more information, read the
//! [Quick Start Guide](https://nita-obel.takulatech.net/learn/quick-start/introduction) for a step-by-step introduction, and [engage with our
//! community](https://nita-obel.takulatech.net/community/) if you have any questions or ideas!
//!
//! ## Example
//! @TODO
//!
//! Don't let the simplicity of the example above fool you. Obel is a [fully featured game engine](https://nita-obel.takulatech.net)
//! and it gets more powerful every day!
//!
//! ## This Crate
//!
//! The `obel` crate is just a container crate that makes it easier to consume Obel subcrates.
//! The defaults provide a "full" engine experience, but you can easily enable / disable features
//! in your project's `Cargo.toml` to meet your specific needs. See Obel's `Cargo.toml` for a full
//! list of features available.
//!
//! If you prefer, you can also consume the individual obel crates directly.
//! Each module in the root of this crate, except for the prelude, can be found on crates.io
//! with `obe_engine_` appended to the front, e.g. `app` -> [`obe_engine_app`](https://docs.rs/nita-obel/*/obe_engine_app/).
#![doc = include_str!("../docs/cargo_features.md")]
#![doc(
    html_logo_url = "https://nita-obel.takulatech.net/assets/icon.png",
    html_favicon_url = "https://nita-obel.takulatech.net/assets/icon.png"
)]

pub use obel_api::*;

// Wasm does not support dynamic linking.
#[cfg(all(feature = "dynamic_linking", not(target_family = "wasm")))]
#[expect(
    unused_imports,
    clippy::single_component_path_imports,
    reason = "This causes obel to be compiled as a dylib when using dynamic linking, and as such cannot be removed or changed without affecting dynamic linking."
)]
use obel_dylib;
