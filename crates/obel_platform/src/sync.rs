//! Provides various synchronization alternatives to language primitives.

pub mod atomic {
    //! Provides various atomic alternatives to language primitives.
    //!
    //! Certain platforms lack complete atomic support, requiring the use of a fallback
    //! such as `portable-atomic`.
    //! Using these types will ensure the correct atomic provider is used without the need for
    //! feature gates in your own code.
    pub use atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16,
        AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering,
    };

    #[cfg(feature = "std")]
    use core::sync::atomic;
    #[cfg(all(not(feature = "std"), feature = "portable-atomic"))]
    use portable_atomic as atomic;
}

pub use arc::{Arc, Weak};

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::sync as arc;
#[cfg(all(not(feature = "std"), not(feature = "std"), feature = "portable-atomic"))]
use portable_atomic_util as arc;
