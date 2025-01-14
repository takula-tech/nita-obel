//! A re-implementation of the currently unstable [`std::sync::Exclusive`]
//!
//! [`std::sync::Exclusive`]: https://doc.rust-lang.org/nightly/std/sync/struct.Exclusive.html

use core::ptr;

/// Normal Sync types like T:Sync and Arc<T> only allows immutable access (read-only) across threads.
/// But `SyncCell` provides a wrapper that allows making any type unconditionally [`Sync`] by only providing mutable access.
/// You can think of it as RWMutex<T> but in complie-time.
///
/// By wrapping a type like a Future in an abstraction like Exclusive,
/// the type is accessed in a thread-safe way - only one thread can mutate it at a time.
///
/// Fo examples, furture and ecs cmpt are only mutably accessed by one thread at any time without data racing concerns.
/// akka, they must satisfy the `sync` trait requirement, in other words, multithreaded safe.
/// So using Mutex is too heavy and damage the perf. SyncCell is more fitting option in this case.
///
///  See [`Exclusive`](https://github.com/rust-lang/rust/issues/98407) for stdlib's upcoming implementation,
/// which should replace this one entirely.
///
#[repr(transparent)]
pub struct SyncCell<T: ?Sized> {
    inner: T,
}

impl<T: Sized> SyncCell<T> {
    /// Construct a new instance of a `SyncCell` from the given value.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Deconstruct this `SyncCell` into its inner value.
    pub fn to_inner(Self { inner }: Self) -> T {
        inner
    }
}

impl<T: ?Sized> SyncCell<T> {
    /// Get a reference to this `SyncCell`'s inner value.
    pub fn get(&mut self) -> &mut T {
        &mut self.inner
    }

    /// For types that implement [`Sync`], get shared access to this `SyncCell`'s inner value.
    pub fn read(&self) -> &T
    where
        T: Sync,
    {
        &self.inner
    }

    /// Build a mutable reference to a `SyncCell` from a mutable reference
    /// to its inner value, to skip constructing with [`new()`](SyncCell::new()).
    pub fn from_mut(r: &mut T) -> &mut SyncCell<T> {
        #[expect(
            unsafe_code,
            reason = "SAFETY:
            repr is transparent, so refs have the same layout.
            and `SyncCell` properties are mut`-agnostic"
        )]
        unsafe {
            &mut *(ptr::from_mut(r) as *mut SyncCell<T>)
        }
    }
}

#[expect(
    unsafe_code,
    reason = "SAFETY:
    `Sync` only allows multithreaded access via immutable reference.
    As `SyncCell` requires an exclusive reference to access the wrapped value for `!Sync` types,
    marking this type as `Sync` does not actually allow un-synchronized access to the inner value"
)]
unsafe impl<T: ?Sized> Sync for SyncCell<T> {}
