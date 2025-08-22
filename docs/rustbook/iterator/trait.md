# `Iterator & IntoIterator`

An iterator is any value that implements the `std::iter::Iterator` trait:

```rust
trait Iterator {
  type Item;
  fn next(&mut self) -> Option<Self::Item>;
  ... // many default methods
  }
```

`Item` is the type of value the iterator produces. The `next`
method either returns `Some(v)`, where v is the iterator’s
next value, or returns `None` to indicate the end of the
sequence. Here we’ve omitted Iterator’s many default
methods; we’ll cover them individually throughout the rest
of this chapter.

If there’s a natural way to iterate over some type, that type
can implement std::iter::IntoIterator, whose `into_iter` method  
takes a value and returns an iterator over it:

```rust
trait IntoIterator
where Self::IntoIter: Iterator<Item=Self::Item> 
{
  type Item;
  type IntoIter: Iterator;
  fn into_iter(self) -> Self::IntoIter;
}
```

`IntoIter` is the type of the iterator value itself, and `Item` is
the type of value it produces. We call any type that
implements IntoIterator an `iterable`, because it’s
something you could iterate over if you asked.

## `Creating Iterators`

### `iter and iter_mut methods`

Each type is free to implement iter and iter_mut in
whatever way makes the most sense for its purpose.

```rust
let v = vec![4, 20];
let mut iterator = v.iter();
assert_eq!(iterator.next(), Some(&4));
assert_eq!(iterator.next(), Some(&20));
assert_eq!(iterator.next(), None);

use std::ffi::OsStr;
use std::path::Path;
let path = Path::new("C:/Users/JimB/Fedora.iso");
let mut iterator = path.iter();
assert_eq!(iterator.next(), Some(OsStr::new("C:")));
assert_eq!(iterator.next(), Some(OsStr::new("Users")));
assert_eq!(iterator.next(), Some(OsStr::new("JimB")));
assert_eq!(iterator.next(), Some(OsStr::new("Fedora.iso")));
assert_eq!(iterator.next(), None);
```

If there’s `more than one common way` to iterate over a
type, the type usually provides specific methods for each
sort of traversal, since a plain iter method would be
ambiguous.  

For example, `s.bytes()` returns an iterator that produces each byte of s,
whereas `s.chars()` interprets the contents as UTF-8 and produces  
each Unicode character.

### `IntoIterator`

When a type implements IntoIterator, you can call its
into_iter method yourself to get an iterator over it.

Rust’s for loop brings all these parts together nicely. To
iterate over a vector’s elements, you can write:

```rust
println!("There's:");
let v = vec!["antimony", "arsenic", "aluminum", "selenium"];
for element in &v {
  println!("{}", element);
}
```

Under the hood, every for loop is just shorthand for calls to
IntoIterator and Iterator methods:

```rust
let mut iterator = (&v).into_iter();
while let Some(element) = iterator.next() {
  println!("{}", element);
}
```

The for loop uses `IntoIterator::into_iter` to convert its
operand `&v` into an iterator and then calls `Iterator::next`
repeatedly. Each time that returns Some(element), the for
loop executes its body; and if it returns None, the loop
finishes.

With this example in mind, here’s some `terminology` for iterators:

- Iterator is any type that implements Iterator
- Iterable is any type that implements IntoIterator
- Iterator produces values
- `Consumer` is the code that receives the produced items by iterator

```rust
(&favorites).into_iter()
for element in &collection { ... }

(&mut favorites).into_iter()
for element in &mut collection { ... }

favorites.into_iter()
for element in collection { ... }
```

### `from_fn and successors`

One simple and general way to produce a sequence of
values is to provide a closure that returns them.

Given a function returning `Option<T>`,
`std::iter::from_fn` returns an iterator that simply calls
the function to produce its items. For example:

```rust
use std::iter::from_fn;
let lengths: Vec<f64> =
from_fn(|| Some((random::<f64>() - random::<f64>()).abs()))
.take(1000)
.collect();
```

This calls `from_fn` to make an iterator producing random
numbers. Since the iterator always returns Some, the
sequence never ends, but we call `take(1000)` to limit it to
the first 1,000 elements. Then `collect` builds the vector
from the resulting iteration. This is an efficient way of
constructing initialized vectors;

If each item depends on the one before, the
`std::iter::successors` function works nicely. You provide
an initial item and a function that takes one item and
returns an Option of the next. If it returns None, the
iteration ends.

```rust
use std::iter::successors;
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
  let zero = Complex { re: 0.0, im: 0.0 };
  successors(Some(zero), |&z| { Some(z * z + c) })
  .take(limit)
  .enumerate()
  .find(|(_i, z)| z.norm_sqr() > 4.0)
  .map(|(i, _z)| i)
}
```
