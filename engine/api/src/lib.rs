//! This module is separated into its own crate to enable simple dynamic linking for Bevy, and should not be used directly

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]

pub use obel_engine_diagnostic as diagnostic;
pub use obel_engine_platform as platform;
pub use obel_engine_reflect as reflect;
