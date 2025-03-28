use alloc::alloc::handle_alloc_error;
use core::{alloc::Layout, cell::UnsafeCell, num::NonZero, ptr::NonNull};
use obel_platform::utils::{OnDrop, OwningPtr, Ptr, PtrMut};

/// A flat, type-erased data storage type
///
/// Used to densely store homogeneous ECS data. A blob is usually just an arbitrary block of contiguous memory without any identity, and
/// could be used to represent any arbitrary data (i.e. string, arrays, etc). This type is an extendable and re-allocatable blob, which makes it a blobby Vec, a `BlobVec`.
pub(super) struct BlobVec {
    item_layout: Layout,
    capacity: usize,
    /// Number of elements, not bytes
    len: usize,
    // the `data` ptr's layout is always `array_layout(item_layout, capacity)`
    data: NonNull<u8>,
    // None if the underlying type doesn't need to be dropped
    drop: Option<unsafe fn(OwningPtr<'_>)>,
}
