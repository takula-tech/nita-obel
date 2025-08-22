# `Generics`

## `Associated Types`

```rust
pub trait Iterator {
  type Item;
  fn next(&mut self) ->Option<Self::Item>
  {
    !unimplemented!()
  }
}
```

The first feature of this trait, `type Item;`, is an associated
type. Each type that implements Iterator `must specify`
what type of item it produces.

The second feature, the `next()` method, uses the
associated type in its return value. next() returns an
`Option<Self::Item>`: either Some(item), the next value in
the sequence, or None when there are no more values to
visit.

Here’s what it looks like to implement Iterator for a type:

```rust
// (code from the std::env standard library module)
impl Iterator for Args {
  type Item = String;
  fn next(&mut self) -> Option<String> {
  ...
  }
  ...
}
```

Generic code can use associated types:

```rust
/// Loop over an iterator, storing the values in a new vector.
fn collect_into_vector<I: Iterator>(iter: I) -> Vec<I::Item> {
  let mut results = Vec::new();
  for value in iter {
    results.push(value);
  }
  results
}
```

let’s look at one more example before moving on:

```rust
/// Print out all the values produced by an iterator
fn dump<I>(iter: I) where I: Iterator
{
  for (index, value) in iter.enumerate() {
    println!("{}: {:?}", index, value); // error
  }
}
```

The gist of the error message is that to make this generic
function compile, we must ensure that `I::Item` implements
the `Debug` trait, the trait for formatting values with `{:?}`. As
the error message suggests, we can do this by placing a
bound on I::Item: use std::fmt::Debug;

```rust
fn dump<I>(iter: I) where I: Iterator, I::Item: Debug
{
  ...
}
```

Or, we could write, “I must be an iterator over Stringv alues”:

```rust
fn dump<I>(iter: I) where I: Iterator<Item=String>
{
  ...
}
```

`Iterator<Item=String>` is itself a trait. If you think of
`Iterator` as the set of all iterator types, then
Iterator<Item=String> is a subset of Iterator: `the set of
iterator types that produce Strings`. This syntax can be  
used anywhere the name of a trait can be used, including  
trait object types:

```rust
fn dump(iter: &mut dyn Iterator<Item=String>) {
  for (index, s) in iter.enumerate() {
    println!("{}: {:?}", index, s);
  }
}
```  

`Associated types` are generally useful whenever a
trait needs to cover more than just methods:

- In a thread pool library, a Task trait, representing a
unit of work, could have an associated Output type.

- A Pattern trait, representing a way of searching a
string, could have an associated Match type,
representing all the information gathered by
matching the pattern to the string:

```rust
trait Pattern {
  type Match;
  fn search(&self, string: &str) ->
  Option<Self::Match>;
}
/// You can search a string for a particular character.
impl Pattern for char {
  /// A "match" is just the location where the character was found.
  type Match = usize;
  fn search(&self, string: &str) -> Option<usize> {
  ...
  }
}
```

If you’re familiar with regular expressions, it’s easy
to see how impl Pattern for RegExp would have a
more elaborate Match type, probably a struct that
would include the start and length of the match, the
locations where parenthesized groups matched, and
so on.

## `Generic Traits`

```rust
/// std::ops::Mul, the trait for types that support `*`.
pub trait Mul<RHS> {
  /// The resulting type after applying the `*` operator
  type Output;
  /// The method for the `*` operator
  fn mul(self, rhs: RHS) -> Self::Output;
}
```

Mul is a generic trait, and its instances `Mul<f64>`, `Mul<String>`, `Mul<Size>`,  
etc., are all `different` traits, just as `min::<i32>` and `min::<String>` are
different functions and `Vec<i32>` and `Vec<String>` are different types.

A single type—say, `WindowSize`—can implement both
`Mul<f64>` and `Mul<i32>`, and many more. You would then
be able to multiply a WindowSize by many other types.
Each implementation would have its own associated `Output` type.

The trait shown earlier is missing one minor detail. The
real Mul trait looks like this:

```rust
pub trait Mul<RHS=Self> {
  ...
}
```

The syntax RHS=Self means that RHS defaults to Self. If I
write `impl Mul for Complex`, without specifying Mul’s type
parameter, it means impl `Mul<Complex> for Complex`. In
a bound, if I write `where T: Mul`, it means `where T:
Mul<T>`.

In Rust, the expression `lhs * rhs` is shorthand for
`Mul::mul(lhs, rhs)`. So overloading the * operator in
Rust is as simple as implementing the `Mul` trait.

## `impl Trait`

As you might imagine, combinations of many generic types
can get messy. For example, combining just a few iterators
using standard library combinators rapidly turns your
return type into an eyesore:

```rust
use std::iter;
use std::vec::IntoIter;
fn cyclical_zip(v: Vec<u8>, u: Vec<u8>) ->
iter::Cycle<iter::Chain<IntoIter<u8>, IntoIter<u8>>> {
  v.into_iter().chain(u.into_iter()).cycle()
}
```

Rust has a feature called impl Trait designed for
precisely this situation. impl Trait allows us to “erase”
the type of a return value, specifying only the trait or traits
it implements, without dynamic dispatch or a heap
allocation:

```rust
fn cyclical_zip(v: Vec<u8>, u: Vec<u8>) -> impl Iterator<Item=u8>
{
  v.into_iter().chain(u.into_iter()).cycle()
}
```

Now, rather than specifying a particular nested type of
iterator combinator structs, cyclical_zip’s signature just
states that it returns some kind of `iterator over u8`. The
return type expresses the `intent` of the function, rather than
its implementation details.

Using `impl Trait` means that you can change
the actual type being returned in the future as long as it
still implements `Iterator<Item=u8>`, and any code calling
the function will continue to compile without an issue. This
provides a lot of flexibility for library authors, because only
the relevant functionality is encoded in the type signature.

## `Associated Consts`

You can declare a trait with an associated
constant using the same syntax as for a struct or enum:

```rust
trait Greet {
  const GREETING: &'static str = "Hello";
  fn greet(&self) -> String;
}
```

Associated consts in traits have a special power, though.
Like associated types and functions, you can declare them
but not give them a value:

```rust
trait Float {
  const ZERO: Self;
  const ONE: Self;
}
```

Then, implementors of the trait can define these values:

```rust
impl Float for f32 {
  const ZERO: f32 = 0.0;
  const ONE: f32 = 1.0;
}

impl Float for f64 {
  const ZERO: f64 = 0.0;
  const ONE: f64 = 1.0;
}
```

This allows you to write generic code that uses these values:

```rust
fn add_one<T: Float + Add<Output=T>>(value: T) -> T {
  value + T::ONE
}
```

Note that associated constants can’t be used with trait
objects, since the compiler relies on type information about
the implementation in order to pick the right value at
compile time.
