//! Platform-agnostic impls to `alloc::borrow` items

#[cfg(feature = "alloc")]
pub use alloc::borrow::ToOwned;

#[cfg(not(feature = "alloc"))]
compile_error!(
    "Missing `borrow` impls in your platform. please report this issue to the maintainer"
);
