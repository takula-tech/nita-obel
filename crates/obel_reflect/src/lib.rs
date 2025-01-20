#![no_std] // tells the compiler "don't automatically link std"
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]

//! obel_reflect

#[cfg(not(feature = "std"))]
extern crate alloc as stdlib;
#[cfg(feature = "std")]
extern crate std as stdlib;

extern crate proc_macro;

mod utils;
pub use utils::*;
