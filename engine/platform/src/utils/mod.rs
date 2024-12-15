//! General utilities for first-party [obel] engine crates.
//!
//! [obel]: https://nita.takulatech.net/docs

pub use default::*;
pub use dropcb::*;
#[cfg(feature = "std")]
pub use parallelqueue::*;
pub use ptr::*;
pub use synccell::*;
pub use syncunsafecell::*;

mod default;
mod dropcb;
#[cfg(feature = "std")]
mod parallelqueue; // thread_local crate needs std
mod ptr;
mod synccell;
mod syncunsafecell;
