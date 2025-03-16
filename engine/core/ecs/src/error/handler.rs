use crate::{error::ObelError, resource::Resource};
use alloc::borrow::Cow;

/// Additional context for a failed system run.
pub struct SystemErrorContext {
    /// The name of the system that failed.
    pub name: Cow<'static, str>,

    /// The last tick that the system was run.
    pub last_run: u32,
}

/// The default systems error handler stored as a resource in the [`World`](crate::world::World).
pub struct DefaultSystemErrorHandler(pub fn(ObelError, SystemErrorContext));

impl Resource for DefaultSystemErrorHandler {}

impl Default for DefaultSystemErrorHandler {
    fn default() -> Self {
        Self(panic)
    }
}

macro_rules! inner {
    ($call:path, $e:ident, $c:ident) => {
        $call!("Encountered an error in system `{}`: {:?}", $c.name, $e);
    };
}

/// Error handler that panics with the system error.
#[track_caller]
#[inline]
pub fn panic(error: ObelError, ctx: SystemErrorContext) {
    inner!(panic, error, ctx);
}

/// Error handler that logs the system error at the `error` level.
#[track_caller]
#[inline]
pub fn error(error: ObelError, ctx: SystemErrorContext) {
    inner!(log::error, error, ctx);
}

/// Error handler that logs the system error at the `warn` level.
#[track_caller]
#[inline]
pub fn warn(error: ObelError, ctx: SystemErrorContext) {
    inner!(log::warn, error, ctx);
}

/// Error handler that logs the system error at the `info` level.
#[track_caller]
#[inline]
pub fn info(error: ObelError, ctx: SystemErrorContext) {
    inner!(log::info, error, ctx);
}

/// Error handler that logs the system error at the `debug` level.
#[track_caller]
#[inline]
pub fn debug(error: ObelError, ctx: SystemErrorContext) {
    inner!(log::debug, error, ctx);
}

/// Error handler that logs the system error at the `trace` level.
#[track_caller]
#[inline]
pub fn trace(error: ObelError, ctx: SystemErrorContext) {
    inner!(log::trace, error, ctx);
}

/// Error handler that ignores the system error.
#[track_caller]
#[inline]
pub fn ignore(_: ObelError, _: SystemErrorContext) {}
