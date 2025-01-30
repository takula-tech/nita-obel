//! Platform-agnostic impls to `string` items

#[cfg(feature = "alloc")]
pub use alloc::{format, string::String, string::ToString};

#[cfg(not(feature = "alloc"))]
compile_error!(
    "Missing `string` impls in your platform. please report this issue to the maintainer"
);
