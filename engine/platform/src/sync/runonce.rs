use crate::sync::atomic::{AtomicBool, Ordering};

/// A thread-safe flag that can be used to ensure code runs exactly once.
///
/// This type wraps an [`AtomicBool`] to provide a simple interface for one-time initialization
/// patterns. It's particularly useful in conjunction with the [`run_once!`] macro for executing
/// code exactly once at a specific call site.
///
/// # Example
/// ```
/// use obel_platform::run_once;
///
/// // This will only print once, even if called multiple times
/// for _ in 0..3 {
///     run_once!(println!("This prints only once"));
/// }
/// ```
pub struct OnceFlag(AtomicBool);

impl OnceFlag {
    /// Creates a new `OnceFlag` in its initial state (unset).
    ///
    /// The flag is initialized to `true`, indicating that the associated code
    /// hasn't been executed yet.
    pub const fn new() -> Self {
        Self(AtomicBool::new(true))
    }

    /// Attempts to set the flag and returns whether the flag was previously unset.
    ///
    /// Returns:
    /// - `true` if this was the first time the flag was set (indicating the associated
    ///   code should run)
    /// - `false` if the flag was already set (indicating the code should be skipped)
    ///
    /// This operation is atomic and thread-safe, using relaxed ordering as the flag
    /// is typically used for one-time initialization that doesn't require synchronization.
    pub fn set(&self) -> bool {
        self.0.swap(false, Ordering::Relaxed)
    }
}

impl Default for OnceFlag {
    fn default() -> Self {
        Self::new()
    }
}

/// A macro that ensures the given expression is executed exactly once per call site.
///
/// This macro is useful for one-time initialization or setup code that should only
/// run once, regardless of how many times the containing code is executed. It's
/// thread-safe and can be used in concurrent contexts.
///
/// # Example
/// ```
/// use obel_platform::run_once;
///
/// fn initialize_resource() {
///     run_once!({
///         // Expensive initialization code here
///         println!("Resource initialized");
///     });
/// }
///
/// // Even if called multiple times, initialization happens only once
/// initialize_resource();
/// initialize_resource();
/// ```
///
/// # Implementation Details
/// The macro creates a static `OnceFlag` that is unique to each macro invocation
/// site. The flag is atomically checked and set, ensuring thread-safe one-time
/// execution of the provided expression.
#[macro_export]
macro_rules! run_once {
    ($expression:expr) => {{
        static SHOULD_FIRE: $crate::sync::OnceFlag = $crate::sync::OnceFlag::new();
        if SHOULD_FIRE.set() {
            $expression;
        }
    }};
}
