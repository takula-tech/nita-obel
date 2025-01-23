//! Platform-agnostic impls to `vec` items

#[cfg(feature = "alloc")]
pub use alloc::vec::*;

#[cfg(not(feature = "alloc"))]
compile_error!("Missing `vec` impls in your platform. please report this issue to the maintainer");
