#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use task::Task;
pub mod futures;

#[cfg_attr(all(target_arch = "wasm32", feature = "web"), path = "wasm_task.rs")]
mod task;

#[cfg(not(feature = "async_executor"))]
mod edge_executor;
mod executor;

cfg_if::cfg_if! {
  if #[cfg(all(not(target_arch = "wasm32"), feature = "multi_threaded"))] {
      pub use mt_task_pool::{Scope, TaskPool, TaskPoolBuilder};
      pub use thread_executor::{ThreadExecutor, ThreadExecutorTicker};
      mod mt_task_pool;
      mod thread_executor;
  } else if #[cfg(any(target_arch = "wasm32", not(feature = "multi_threaded")))] {
    pub use st_task_pool::{Scope, TaskPool, TaskPoolBuilder, ThreadExecutor};
    mod st_task_pool;
  }
}

mod task_pool_macro;
pub use futures_lite::future::poll_once;
#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
pub use task_pool_macro::tick_global_task_pools_on_main_thread;
pub use task_pool_macro::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool};

#[cfg(feature = "std")]
cfg_if::cfg_if! {
    if #[cfg(feature = "async-io")] {
        pub use async_io::block_on;
    } else {
        pub use futures_lite::future::block_on;
    }
}

mod parallel;
pub use parallel::{ParallelSlice, ParallelSliceMut};

/// The tasks prelude.
///
/// This includes the most common types in this crate, re-exported for your convenience.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        parallel::{ParallelIterator, ParallelSlice, ParallelSliceMut},
        task_pool_macro::{AsyncComputeTaskPool, ComputeTaskPool, IoTaskPool},
    };

    #[cfg(feature = "std")]
    #[doc(hidden)]
    pub use crate::block_on;
}

cfg_if::cfg_if! {
  if #[cfg(feature = "std")] {
      use core::num::NonZero;

      /// Gets the logical CPU core count available to the current process.
      ///
      /// This is identical to [`std::thread::available_parallelism`], except
      /// it will return a default value of 1 if it internally errors out.
      ///
      /// This will always return at least 1.
      pub fn available_parallelism() -> usize {
          std::thread::available_parallelism()
              .map(NonZero::<usize>::get)
              .unwrap_or(1)
      }
  } else {
      /// Gets the logical CPU core count available to the current process.
      ///
      /// This will always return at least 1.
      pub fn available_parallelism() -> usize {
          // Without access to std, assume a single thread is available
          1
      }
  }
}
