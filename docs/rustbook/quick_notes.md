## pub(crate)

In Rust, pub(crate) is a visibility modifier that makes an item public only within the current crate. This means the item can be accessed from any module inside the crate but not from outside the crate.

## Syn Span

Instead of using `Span::call_site()` (which always generates a span at the macro invocation site), you can extract the span from the parent struct's name.

This makes sure the generated fields and methods inherit the same span as the struct itself. This is useful for:

- Better Error Messages → Errors will point to the struct name, not just the macro call.
- Correct Scope Handling → The field will be properly tied to the struct’s scope.
- Code Readability → IDEs and linters can track where the code was generated.

## 'static lifetime

Common Cases Where 'static lifetime is Used

- Global Data: E.g., &'static str (string literals are stored in the binary).
- Owned Types: Like String, Vec<T>, or any type that doesn’t borrow data.
- Static Singletons: E.g., Mutex<()> or AtomicBool.
- Closures: 'static closures (they don’t capture references to short-lived variables).
- pub trait Resource: Send + Sync + 'static {}
