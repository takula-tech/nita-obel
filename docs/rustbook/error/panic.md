# `Panic`

panic represents a bug in the program caused by programmer's faults.
it is like logic_error in c++:

- Out-of-bounds array access
- Integer division by zero
- Calling .expect() on a Result that happens to be Err
- Assertion failure

the rule of thumb is don't panic. But we all make mistakes.
so when a panic does occur, Rust can either unwind the stack
or abort the process where the unwinding is the default behavior.

## `Unwinding`

Any local variables and arguments
that the current function was using are dropped, in
the reverse of the order they were created. user-defined drop
method is called too. then move the caller and unwind in the same way.
and so on up the stack.

Finally, the thread exits. If the panicking thread was
the main thread, then the whole process exits (with
a nonzero exit code).

## `Aborting`

Rust will abort the program in the cases:

- drop method triggers a second panic when a thread is unwinding the first panic.
- user configures the complier option -C panic=abort to make abort as default panic behavior

