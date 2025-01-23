//! Platform agnostic support

// when docsrs flag is enabled, doc_auto_cfg feature is activated,
// enriching the documentation with conditional information.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
pub mod collections;
pub mod hash;
pub mod string;
pub mod sync;
pub mod time;
pub mod utils;
pub mod vec;
