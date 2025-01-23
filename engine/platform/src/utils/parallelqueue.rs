use alloc::vec::Vec;
use core::{cell::RefCell, ops::DerefMut};
use thread_local::ThreadLocal;

/// A thread-safe collection that maintains separate instances of a value for each thread.
/// This structure is particularly useful for scenarios where you need thread-local storage
/// with the ability to mutably access values. For types implementing `Default`, mutable
/// references can be obtained through [`Parallel::scope`].
///
/// If you do need to share data across threads, you should use synchronization
/// primitives like [`std::sync::Mutex`], [`crate::sync_cell::SyncCell`]
/// [`std::sync::RwLock`], but Parallel<T> is specifically designed for cases where you want
/// to avoid synchronization overhead by giving each thread its own copy.
#[derive(Default)]
pub struct Parallel<T: Send> {
    locals: ThreadLocal<RefCell<T>>,
}

/// Core implementation for any Send type
impl<T: Send> Parallel<T> {
    /// Provides a mutable iterator over all thread-local values.
    /// This allows you to modify each thread's local value in sequence.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.locals.iter_mut().map(RefCell::get_mut)
    }

    /// Removes all thread-local values from the collection.
    /// After this operation, the next access to any thread-local value will create a new one.
    pub fn clear(&mut self) {
        self.locals.clear();
    }
}

impl<T: Default + Send> Parallel<T> {
    /// Safely accesses and modifies the current thread's local value through a closure.
    ///
    /// If no value exists for the current thread, one is created using `Default::default()`.
    /// The provided closure receives a mutable reference to the thread-local value and can
    /// return any type.
    pub fn scope<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut cell = self.locals.get_or_default().borrow_mut();
        let ret = f(cell.deref_mut());
        ret
    }

    /// Provides direct mutable access to the current thread's local value.
    ///
    /// Creates a new value using `Default::default()` if none exists for the current thread.
    /// Returns a guard that implements `DerefMut`, allowing safe mutable access to the value.
    pub fn borrow_local_mut(&self) -> impl DerefMut<Target = T> + '_ {
        self.locals.get_or_default().borrow_mut()
    }
}

impl<T, I> Parallel<I>
where
    I: IntoIterator<Item = T> + Default + Send + 'static,
{
    /// Creates an iterator that consumes and yields all items from all thread-local collections.
    ///
    /// Unlike [`Vec::drain`], this method processes the data in chunks. If iteration is
    /// stopped early, any remaining items in the current chunk are dropped, while items
    /// in unprocessed chunks remain in their original thread-local collections.
    ///
    /// Note: The order of items in the resulting iterator is not deterministic.
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.locals.iter_mut().flat_map(|item| item.take())
    }
}

impl<T: Send> Parallel<Vec<T>> {
    /// Efficiently moves all items from all thread-local vectors into a single target vector.
    ///
    /// This method:
    /// 1. Pre-allocates space in the target vector to avoid repeated reallocations
    /// 2. Moves items using `Vec::append` for optimal performance
    /// 3. Preserves the capacity of the source vectors for future use
    ///
    /// Note: The order of items in the resulting vector is not deterministic.
    pub fn drain_into(&mut self, out: &mut Vec<T>) {
        let size = self.locals.iter_mut().map(|queue| queue.get_mut().len()).sum();
        out.reserve(size);
        for queue in self.locals.iter_mut() {
            out.append(queue.get_mut());
        }
    }
}
