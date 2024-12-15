#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(
    html_logo_url = "https://obelengine.org/assets/icon.png",
    html_favicon_url = "https://obelengine.org/assets/icon.png"
)]

//! Forces dynamic linking of obel.
//!
//! Dynamic linking causes obel to be built and linked as a dynamic library. This will make
//! incremental builds compile much faster.
//!
//! # Warning
//!
//! Do not enable this feature for release builds because this would require you to ship
//! `libstd.so` and `libobel_dylib.so` with your game.
//!
//! # Enabling dynamic linking
//!
//! ## The recommended way
//!
//! The easiest way to enable dynamic linking is to use the `--features obel/dynamic_linking` flag when
//! using the `cargo run` command:
//!
//! `cargo run --features obel/dynamic_linking`
//!
//! ## The unrecommended way
//!
//! It is also possible to enable the `dynamic_linking` feature inside of the `Cargo.toml` file. This is
//! unrecommended because it requires you to remove this feature every time you want to create a
//! release build to avoid having to ship additional files with your game.
//!
//! To enable dynamic linking inside of the `Cargo.toml` file add the `dynamic_linking` feature to the
//! obel dependency:
//!
//! `features = ["dynamic_linking"]`
//!
//! ## The manual way
//!
//! Manually enabling dynamic linking is achieved by adding `obel_dylib` as a dependency and
//! adding the following code to the `main.rs` file:
//!
//! ```
//! #[allow(unused_imports)]
//! use obel_dylib;
//! ```
//!
//! It is recommended to disable the `obel_dylib` dependency in release mode by adding the
//! following code to the `use` statement to avoid having to ship additional files with your game:
//!
//! ```
//! #[allow(unused_imports)]
//! #[cfg(debug_assertions)] // new
//! use obel_dylib;
//! ```

// Force linking of the main obel crate
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use obel_api;
