//! General utilities for first-party [obel] engine crates.
//!
//! [obel]: https://nita.takulatech.net/docs

// when docsrs flag is enabled, doc_auto_cfg feature is activated,
// enriching the documentation with conditional information.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]

#[cfg(feature = "tracing")]
pub use ::tracing;
#[cfg(feature = "tracing")]
mod trace;

#[cfg(any(feature = "std", target_arch = "wasm32"))]
mod time;
#[cfg(any(feature = "std", target_arch = "wasm32"))]
pub use time::*;

mod struct_default;
pub use struct_default::*;

#[cfg(feature = "alloc")]
mod hashing;
#[cfg(feature = "alloc")]
pub use hashing::*;

mod conditionals;
pub use conditionals::*;

mod futures;
pub use futures::*;

mod object_safe;
pub use object_safe::*;

mod drop_cb;
pub use drop_cb::*;

#[cfg(feature = "std")]
mod parallel_queue;
#[cfg(feature = "std")]
pub use parallel_queue::*;

mod sync_cell;
pub use sync_cell::*;

mod sync_cell_unsafe;
pub use sync_cell_unsafe::*;
