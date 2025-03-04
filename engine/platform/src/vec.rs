//! Platform-agnostic impls to `vec` items
//! @TODO: add stack-alloc impls of vec for embedded system without heap allocations.

#[cfg(feature = "alloc")]
pub use alloc::vec::Vec;

#[cfg(not(feature = "alloc"))]
compile_error!("Missing `vec` impls in your platform. please report this issue to the maintainer");
