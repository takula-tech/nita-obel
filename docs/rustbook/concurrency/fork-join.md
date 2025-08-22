# `Fork-Join`

```rust
use std::{ thread, io };
fn process_files(file_names: Vec<string>) -> io::Result<()> {
  for document in file_names {
    let text = load(&document)?;
    let results = parse(text);
    save(&document, results)?;
  }
  Ok(())
}
```

fork-join approach can speed up the isolated units of work by splitting the tasks into
smaller chunks and processing each chunk on a separate thread.

It has the following benefits:

- no locking of the shared data and so the performance is better
- easy to reason about the program correctness as the same inputs always result in same outputs

## `spawn and join`

```rust
use std::thread::{thread, io};
fn process_files(file_names: Vec<string>) -> io::Result<()> {
  // divide the work unit
  const NTHREADS = 8_usize;
  let chunks = split_into_chunks(file_names, NTHREADS);
  // process each chunk in a separate thread
  let mut thread_handles = vec![];
  for chunk in chunks {
    let handle = thread::spawn(move || process_file(chunk));
    thread_handles.push(handle);
  }
  // wait for all the threads to finish
  for handle in thread_handles {
    handle.join()?;
  }
  Ok(())
}
```

`Joining threads` is often necessary for correctness, because
a Rust program exits as soon as main returns, even if other
threads are still running. the extra threads are just killed
and destructors are not called.


## `Error Handling across threads`

In Rust, panic is safe and per thread. The
boundaries between threads serve as a firewall for panic;

panic doesn’t automatically spread from one thread to the
threads that depend on it. Instead, a panic in one thread is
reported as an error Result in other threads. The program
as a whole can easily recover

```rust
let handle_panic = thread::spawn(|| worker(true, false));
match handle_panic.join() {
    Ok(Ok(val)) => println!("Success: {}", val),
    Ok(Err(e)) => println!("Application error: {:?}", e),
    Err(panic) => println!("Thread panicked: {:?}", panic),
}
```

