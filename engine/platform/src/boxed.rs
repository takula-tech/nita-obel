//! Platform-agnostic impls to `alloc::boxed` items

#[cfg(feature = "alloc")]
pub use alloc::boxed::Box;

#[cfg(not(feature = "alloc"))]
compile_error!("Missing `Box` impls in your platform. please report this issue to the maintainer");
