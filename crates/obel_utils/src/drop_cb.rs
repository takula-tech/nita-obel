//!  Utilities to call fn when dropped

#[cfg(feature = "std")]
extern crate std;

use core::mem::ManuallyDrop;

/**
 A type which calls a function when dropped.
This can be used to ensure that cleanup code is run even in case of a panic.

Note that this only works for panics that [unwind](https://doc.rust-lang.org/nomicon/unwinding.html)
-- any code within `OnDrop` will be skipped if a panic does not unwind.
In most cases, this will just work.

# Examples

```rust
# use obel_utils::OnDrop;
# fn test_panic(do_panic: bool, log: impl FnOnce(&str)) {
// This will print a message when the variable `_catch` gets dropped,
// even if a panic occurs before we reach the end of this scope.
// This is similar to a `try ... catch` block in languages such as C++.
let _catch = OnDrop::new(|| log("Oops, a panic occurred and this function didn't complete!"));

// Some code that may panic...
// ...
# if do_panic { panic!() }

// Make sure the message only gets printed if a panic occurs.
// If we remove this line, then the message will be printed regardless of whether a panic occurs
// -- similar to a `try ... finally` block.
core::mem::forget(_catch);
# }
#
# test_panic(false, |_| unreachable!());
# let mut did_log = false;
# std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
#   test_panic(true, |_| did_log = true);
# }));
# assert!(did_log);
```
*/
pub struct OnDrop<F: FnOnce()> {
    callback: ManuallyDrop<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    ///  Returns an object that will invoke the specified callback when dropped.
    pub fn new(callback: F) -> Self {
        Self {
            callback: ManuallyDrop::new(callback),
        }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        #[expect(
            unsafe_code,
            reason = "SAFETY: We may move out of `self`, since this instance can never be observed after it's dropped"
        )]
        let callback = unsafe { ManuallyDrop::take(&mut self.callback) };
        callback();
    }
}
