//! obel_reflect

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

extern crate proc_macro;

#[cfg(feature = "std")]
mod utils;
#[cfg(feature = "std")]
pub use utils::*;
