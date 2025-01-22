//! General utilities for first-party [obel] engine crates.
//!
//! [obel]: https://nita.takulatech.net/docs

// when docsrs flag is enabled, doc_auto_cfg feature is activated,
// enriching the documentation with conditional information.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod struct_default;
pub use struct_default::*;

#[cfg(feature = "alloc")]
pub use hashbrown;

mod hashing;
#[cfg(feature = "alloc")]
pub use hashing::alloc_mod::*;
pub use hashing::common_mod::*;

mod drop_cb;
pub use drop_cb::*;

#[cfg(feature = "alloc")]
mod parallel_queue;
#[cfg(feature = "alloc")]
pub use parallel_queue::*;

mod sync_cell;
pub use sync_cell::*;

mod sync_cell_unsafe;
pub use sync_cell_unsafe::*;

mod once;
pub use once::*;
