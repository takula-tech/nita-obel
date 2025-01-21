#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std]

//! Platform-agnostic types and api

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

pub mod sync;
pub mod time;
