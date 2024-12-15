//! Utilities for working with [`Future`]s.

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

/// Consumes a future, polls it once, and immediately returns the
/// output  or returns `None` if it wasn't ready yet.
/// This will cancel the future if it's not ready.
pub fn now_or_never<F: Future>(mut future: F) -> Option<F::Output> {
    let noop_waker = noop_waker();
    let mut cx = Context::from_waker(&noop_waker);
    // SAFETY: `future` is not moved and the original value is shadowed
    #[expect(unsafe_code)]
    let future = unsafe { Pin::new_unchecked(&mut future) };
    match future.poll(&mut cx) {
        Poll::Ready(x) => Some(x),
        _ => None,
    }
}

/// Polls a future once, and returns the output if ready
/// or returns `None` if it wasn't ready yet.
pub fn check_ready<F: Future + Unpin>(future: &mut F) -> Option<F::Output> {
    let noop_waker = noop_waker();
    let mut cx = Context::from_waker(&noop_waker);
    let future = Pin::new(future);
    match future.poll(&mut cx) {
        Poll::Ready(x) => Some(x),
        _ => None,
    }
}

#[inline]
fn noop_raw_waker() -> RawWaker {
    RawWaker::new(core::ptr::null(), &NOOP_WAKER_VTABLE)
}
#[inline]
fn noop_clone(_data: *const ()) -> RawWaker {
    noop_raw_waker()
}
#[inline]
fn noop(_data: *const ()) {}
const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);
#[inline]
fn noop_waker() -> Waker {
    // SAFETY: the `RawWakerVTable` is just a big noop and doesn't violate any of the rules in `RawWakerVTable`s documentation
    // (which talks about retaining and releasing any "resources", of which there are none in this case)
    #[expect(unsafe_code)]
    unsafe {
        Waker::from_raw(noop_raw_waker())
    }
}
