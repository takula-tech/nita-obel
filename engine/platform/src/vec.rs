//! Platform-agnostic impls to `vec` items

#[cfg(feature = "alloc")]
pub use alloc::vec::Vec;

#[cfg(not(feature = "alloc"))]
compile_error!("Missing `vec` impls in your platform. please report this issue to the maintainer");
