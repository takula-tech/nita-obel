# `Result`

Result type represents ordinary errors caused by things
`outside the program`, like invalid user input, a network
outage, or a permissions problem.

those errors occur is out of control of programmers.
even a bug-free program will encounter those errors from time to time.

## `Catching Errors`

```rust
match result {
  Ok(value) => /* code */,
  Err(error) => /* code */,
}

// The methods listed here are the ones we use the most:
result.is_ok(); // bool
result.is_err(); // bool
result.as_ref(); //Result<&T, &E>
result.as_mut(); // Resul <&mut T, &mut E>
result.unwrap(); // T
result.expect("error: {:?}", err);// T with context infos
result.unwrap_or(default); // T with default value
result.unwrap_or_else(|err| { println!("error: {:?}", err); default})
result.ok() // Option<T>: Some(value) or None
result.err() // Option<E>: Some(error) or None
```

## `Printing Errors`

Sometimes the only way to handle an error is by dumping it
to the terminal and moving on. We already showed one way
to do this:

```rust
println!("error querying the weather: {}", err);
```

The standard library defines several error types with
boring names: `std::io::Error`, `std::fmt::Error`,
`std::str::Utf8Error`, and so on.All of them implement a
common interface, the `std::error::Error` trait, which
means they share the following features and methods:

- println!()

  ```rust
  // result of `println!("error: {}", err);`
  error: failed to lookup address information: No address associated with hostname

  // result of `println!("error: {:?}", err);`
  error: Error { 
    repr: Custom(Custom { 
      kind: Other, 
      error: StringError("failed to lookup address information:
      No address associated with hostname")
    }) 
  }
  ```

- err.to_string() / err.source()
  If err.to_string() is `"boat was repossessed"`, then err.source() might
  return an error about the failed transaction. That error’s
  .to_string() might be `"failed to transfer $300 to United Yacht Supply"`,
  and its .source() might be an `io::Error` with details about the specific network
  outage that caused all the fuss. This third error is the
  root cause, so its .source() method would return None.

Printing an error value does not also print out its source. If
you want to be sure to print all the available information,
use this function:

```rust
use std::error::Error;
use std::io::{Write, stderr};
/// Dump an error message to `stderr`.
///
/// If another error happens while building the error message or
/// writing to `stderr`, it is ignored.
fn print_error(mut err: &dyn Error) {
  let _  = writeln!(stderr(), "error: {}", err);
  while let Some(source) = err.source() {
    let _ = writeln!(stderr(), "caused by: {}", source);
    err = source;
  }
}
```

## `Propagating errors`

```rust
use std::fs;
use std::io;
use std::path::Path;
fn move_all(src: &Path, dst: &Path) -> io::Result<()> {
  for entry_result in src.read_dir()? { // opening dir could fail
    let entry = entry_result?; // reading dir could fail
    let dst_file = dst.join(entry.file_name());
    fs::rename(entry.path(), dst_file)?; // renaming could fail
  }
  Ok(()) // phew!
}
```

? also works similarly with the `Option` type. In a function
that returns Option, you can use ? to unwrap a value and
return early in the case of None:

```rust
let weather = get_weather(hometown).ok()?;
```

## `Multiple Error Types`

```rust
use std::error::Error;
use std::result::Result;
pub type WhateverError = Box<dyn Error + Send + Sync +'static>;
pub type WhateverResult<T> = Result<T, WhateverError>;

let io_error = io::Error::new(io::ErrorKind::Other, "timed out");
return Err(WhateverError::from(io_error)); // manually convert to GenericError
```

If you’re calling a function that returns a GenericResult
and you want to handle one particular kind of error but let
all others propagate out, use the generic method
`error.downcast_ref::<ErrorType>()`:

```rust
loop {
  match compile_project() {
    Ok(()) => return Ok(()),
    Err(err) => {
      if let Some(mse) = err.downcast_ref::<MissingSemicolonError>() {
        insert_semicolon_in_source_code(mse.file(), mse.line())?;
        continue; // try again!
      }
      return Err(err);
    }
  }
}
```









## `Ignoring errors`

```rust
writeln!(stderr(), "error: {}", err); // warning: unused result
// The idiom let _ = ... is used to silence this warning:
let _ = writeln!(stderr(), "error: {}", err); // ok, ignore result
```