//! Collection types and utilities for working with collections in a platform-agnostic way.

#[cfg(feature = "alloc")]
pub use hashbrown::*;

#[cfg(feature = "alloc")]
mod hashbrown;
