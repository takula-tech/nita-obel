//! This module is separated into its own crate to enable simple dynamic linking for Obel, and should not be used directly

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"

pub use obel_diagnostic as diagnostic;
pub use obel_platform as platform;
pub use obel_reflect as reflect;
