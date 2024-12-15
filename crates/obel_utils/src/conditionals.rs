//!  Utilities  for working with conditional_send

pub use conditional_send::*;

#[cfg(not(target_arch = "wasm32"))]
mod conditional_send {
    /// For any type T that already implements [`Send`],
    /// the trait [`ConditionalSend`] is also implemented for that type.
    /// Use [`ConditionalSend`] for a future with an optional Send trait bound
    /// on certain platforms (eg. Wasm) where the futures aren't Send.
    pub trait ConditionalSend: Send {}
    impl<T: Send> ConditionalSend for T {}
}

#[cfg(target_arch = "wasm32")]
#[expect(missing_docs, reason = "Not all docs are written yet (#3492).")]
mod conditional_send {
    pub trait ConditionalSend {}
    impl<T> ConditionalSend for T {}
}

/// For any type T that already implements [`core::future::Future`]+[`ConditionalSend`],
/// the trait [`ConditionalSendFuture`] is also implemented for that type.
/// Use [`ConditionalSendFuture`] for a future with an optional Send trait bound
/// on certain platforms (eg. Wasm) where the futures aren't Send.
pub trait ConditionalSendFuture: core::future::Future + ConditionalSend {}
impl<T: core::future::Future + ConditionalSend> ConditionalSendFuture for T {}

#[cfg(feature = "alloc")]
mod boxed_con_fut {
    extern crate alloc;
    use alloc::boxed::Box;
    /// An owned and dynamically typed Future used when you can't statically type your result
    /// or need to add some indirection.
    pub type BoxedConditionalFuture<'a, T> =
        core::pin::Pin<Box<dyn super::ConditionalSendFuture<Output = T> + 'a>>;
}
#[cfg(feature = "alloc")]
pub use boxed_con_fut::*;
