//! Provides various synchronization alternatives to language primitives.
//!
//! Currently missing from this module are the following items:
//! * `Condvar`
//! * `WaitTimeoutResult`
//! * `mpsc`
//!
//! Otherwise, this is a drop-in replacement for `std::sync`.

pub mod atomic;
#[cfg(feature = "alloc")]
pub use arc::{Arc, Weak};
pub use barrier::{Barrier, BarrierWaitResult};
pub use lazylock::LazyLock;
pub use mutex::{Mutex, MutexGuard};
pub use once::{Once, OnceLock, OnceState};
pub use poison::{LockResult, PoisonError, TryLockError, TryLockResult};
pub use runonce::OnceFlag;
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};

mod barrier;
mod lazylock;
mod mutex;
mod once;
mod poison;
mod runonce;
mod rwlock;

#[cfg(all(feature = "alloc", not(target_has_atomic = "ptr")))]
use portable_atomic_util as arc;

#[cfg(all(feature = "alloc", target_has_atomic = "ptr"))]
use alloc::sync as arc;
