# `Utility Traits`

| Trait                        | Description                                                                                                            |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| **Drop**                     | **Destructors.** Cleanup code that Rust runs automatically whenever a value is dropped.                                |
| **Sized**                    | **Marker trait** for types with a fixed size known at compile time, unlike dynamically sized types.                    |
| **Clone**                    | Types that support cloning values.                                                                                     |
| **Copy**                     | **Marker trait** for types that can be cloned simply by making a byte-for-byte copy of the memory.                     |
| **Deref** and **DerefMut**   | Traits for smart pointer types.                                                                                        |
| **Default**                  | Types that have a sensible “default value.”                                                                            |
| **AsRef** and **AsMut**      | **Conversion traits** for borrowing one type of reference from another.                                                |
| **Borrow** and **BorrowMut** | **Conversion traits**, like `AsRef`/`AsMut`, but additionally guaranteeing consistent hashing, ordering, and equality. |
| **From** and **Into**        | **Conversion traits** for transforming one type of value into another.                                                 |
| **TryFrom** and **TryInto**  | **Conversion traits** for transforming one type of value into another, for transformations that might fail.            |
| **ToOwned**                  | **Conversion trait** for converting a reference to an owned value.                                                     |

## `Drop`

When a value is dropped, if it implements `std::ops::Drop`,
Rust calls its drop method, `before` proceeding to drop whatever
values its fields or elements own, as it normally would.
This implicit invocation of drop is the only way to call that method;
if you try to invoke it `explicitly` yourself, Rust flags that as an `error`.

If a type implements Drop, it cannot implement the Copy
trait. The standard prelude includes a function to drop a value,
drop, but its definition is anything but magical:
```rust fn drop<T>(_x: T) { }```

## Sized

The only use for Sized is as a bound for type variables:
a bound like `T: Sized` requires T to be a type whose size is `known at compile time`.
Traits of this sort are called marker traits, because the Rust language itself uses
them to mark certain types as having characteristics of interest.

However, Rust also has a few `unsized` types whose values
are not all the same size. For example, the string slice type
`str` (note, without an &) is unsized.

Rust can’t store unsized values in variables or pass them as
arguments. You can only deal with them through pointers
like &str or Box<dyn Write>, which themselves are sized.

<img src="unsized.png" />

In fact, it is the implicit default in Rust: if
you write struct S<T> { ... }, Rust understands you to
mean struct S<`T: Sized`> { ... }. If you do not want to
constrain T this way, you must explicitly opt out, writing
struct S<`T: ?Sized`> { ... }.

For example, if you write:

```rust
struct S<T: ?Sized> {
  b: Box<T>
}
```

then Rust will allow you to write `S<str>` and `S<dyn Write>`,
where the box becomes a fat pointer, as well as S<i32> and S<String>,
where the box is an ordinary pointer.

you can use this RcBox with sized types, like `RcBox<String>`;
the result is a `sized` struct type. Or you can use it with
unsized types, like `RcBox<dyn std::fmt::Display>`,
RcBox<dyn Display> is an `unsized` struct type

```rust
struct RcBox<T: ?Sized> {
  ref_count: usize,
  value: T,
}
```

## `Clone`

```rust
trait Clone: Sized {
fn clone(&self) -> Self;
  fn clone_from(&mut self, source: &Self) {
    *self = source.clone()
  }
}
```

In generic code, you should use `clone_from`
whenever possible to take advantage of optimized
implementations when present. eg:

if the string s heap buffer belonging to the original s has
enough capacity to hold t’s contents, no allocation or deallocation is necessary:
you can simply copy t’s text into s’s buffer and adjust the length.

Some types don’t make sense to copy, like
`std::sync::Mutex`; those don’t implement Clone.
Some types like `std::fs::File` can be copied, but the copy
might `fail` if the operating system doesn’t have the
necessary resources; these types don’t implement Clone,
since clone must be infallible. Instead, std::fs::File
provides a `try_clone` method, which returns a
std::io::Result<File>, which can report a failure.

If your Clone implementation simply applies clone to each
field or element of your type and then constructs a new
value from those clones, and the default definition of
clone_from is good enough, then Rust will implement that
for you: simply put `#[derive(Clone)]` above your type
definition.

## Copy

Copy is a `marker trait` with special meaning to
the language. ```rust trait Copy: Clone { }```

This is certainly easy to implement for your own types:
```rust impl Copy for MyType { }```

Rust permits a type to implement Copy only if
a shallow `byte-for-byte copy` is all it needs.

Types that own any other resources, like heap buffers or
operating system handles, cannot implement Copy.
Any type that implements the Drop trait cannot be Copy.

## `Deref and DerefMut`

The traits are defined like this:

```rust
trait Deref {
  type Target: ?Sized;
  fn deref(&self) -> &Self::Target;
}
trait DerefMut: Deref {
  fn deref_mut(&mut self) -> &mut Self::Target;
}
```

For example, suppose you have the following type:

```rust
struct Selector<T> {
  /// Elements available in this `Selector`.
  elements: Vec<T>,
  /// The index of the "current" element in `elements`. A
  `Selector`
  /// behaves like a pointer to the current element.
  current: usize
}
```

To make the Selector behave as the doc comment claims,
you must implement Deref and DerefMut for the type:

```rust
use std::ops::{Deref, DerefMut};
impl<T> Deref for Selector<T> {
  type Target = T;
  fn deref(&self) -> &T {
    &self.elements[self.current]
  }
}
impl<T> DerefMut for Selector<T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut self.elements[self.current]
  }
}
```

Given those implementations, you can use a Selector like this:

```rust
let mut s = Selector { elements: vec!['x', 'y', 'z'], current: 2 };
// Because `Selector` implements `Deref`, we can use the `*`
// operator to  refer to its current element.
assert_eq!(*s, 'z');

// Assert that 'z' is alphabetic, using a method of `char`
// directly on a `Selector`, via deref coercion.
assert!(s.is_alphabetic());

// Change the 'z' to a 'w', by assigning to the `Selector`'s referent.
*s = 'w';
assert_eq!(s.elements, ['x', 'y', 'w']);
```

For example, the following code works fine:

```rust
let s = Selector { elements: vec!["good", "bad", "ugly"],current: 2 };
fn show_it(thing: &str) { println!("{}", thing); }
show_it(&s);
```

In the call show_it(&s), Rust sees an argument of type
`&Selector<&str>` and a parameter of type `&str`, finds the
`Deref<Target=str>` implementation, and rewrites the call
to `show_it(s.deref())`, just as needed.

However, if you change show_it into a `generic` function,
Rust is suddenly no longer cooperative:

```rust
use std::fmt::Display;
fn show_it_generic<T: Display>(thing: T) { println!("{}", thing);}
show_it_generic(&s);

Rust complains:
error: `Selector<&str>` doesn't implement `std::fmt::Display`
|
33 | fn show_it_generic<T: Display>(thing: T) { println!("
{}", thing); }
| ------- required by this bound in
| `show_it_generic`
34 | show_it_generic(&s);
| ^^
| |
| `Selector<&str>` cannot be formatted
with the
`&*s`
|
| default formatter
| help: consider adding dereference here:
`&*s`
|
```

Selector<&str> does not implement Display itself,
but it `dereferences` to &str, which certainly does.
Rust checks whether the bound `T: Display` is satisfied:
since it does not apply deref coercions to satisfy bounds
on `type variables`, this check fails.

To work around this problem, you can spell out the
coercion using the `as` operator: ```show_it_generic(&s as &str);```

Or, as the compiler suggests, you can force the coercion
with `&*`: ```show_it_generic(&*s);```

The Deref and DerefMut traits are `designed` for
implementing `smart pointer types`, like Box, Rc, and Arc,
and types that serve as `owning` versions of `something` you
would also frequently use by reference, the way Vec<T> and
String serve as owning versions of [T] and str.

## `Default`

Some types have a reasonably obvious default value: the
default vector or string is empty, the default number is
zero, the default Option is None, and so on. Types like this
can implement the std::default::Default trait:

```rust
trait Default {
  fn default() -> Self;
}
```

The default method simply returns a fresh value of type
Self. String’s implementation of Default is
straightforward:

```rust
impl Default for String {
  fn default() -> String {
    String::new()
  }
}
```

Another common use of Default is to produce default
values for `structs` that represent a `large` collection of
parameters, most of which you won’t usually need to
change.

The glium draw function expects a `DrawParameters struct` as an argument.
Since DrawParameters implements Default, you can create
one to pass to draw, mentioning only those fields you want
to change:

```rust
let params = glium::DrawParameters {
  line_width: Some(0.02),
  point_size: Some(0.02),
  .. Default::default()
};
target.draw(..., &params).unwrap();
```

If a type T implements Default, then the standard library
implements Default `automatically` for:  

- Rc<T>, Arc<T>, Box<T>
- Cell<T>, RefCell<T>
- Mutex<T>, RwLock<T>
- Cow<T>

The default value for the type Rc<T>,
for example, is an Rc pointing to the default value for type T.

If all the element types of a `tuple` type implement Default,
then the tuple type does too, defaulting to a tuple holding
each element’s default value.

Rust does not `implicitly` implement Default for struct
types, but if all of a struct’s fields implement Default, you
can implement Default for the struct automatically using `#[derive(Default)]`.

## `AsRef and AsMut`

```rust
trait AsRef<T: ?Sized> {
  fn as_ref(&self) -> &T;
}
trait AsMut<T: ?Sized> {
  fn as_mut(&mut self) -> &mut T;
}
```

Vec<T> implements AsRef<[T]>, and String implements AsRef<str>.  
You can also borrow a String’s contents as an array of bytes,
so String implements AsRef<[u8]> as well.

AsRef is typically used to make functions more flexible in
the argument types they accept. For example, the
std::fs::File::open function is declared like this:
```fn open<P: AsRef<Path>>(path: P) -> Result<File>```

What open really wants is a &Path, the type representing a
filesystem path. But with this signature, open accepts
anything it can borrow a &Path from—that is, anything that
implements `AsRef<Path>`. Such types include String and
str, the operating system interface string types OsString
and OsStr, and of course PathBuf and Path;

For callers, the eﬀect resembles that of an overloaded function in C++,
although Rust takes a different approach toward establishing which
argument types are acceptable.


## `Borrow and BorrowMut`

Borrow’s definition is identical to that of AsRef; only the
names have been changed:

```rust
trait Borrow<Borrowed: ?Sized> {
  fn borrow(&self) -> &Borrowed;
}
```

Borrow is designed to address a specific situation with
generic hash tables and other associative collection types.
For example, suppose you have a `std::collections
::HashMap<String, i32>`, mapping strings to numbers.
This table’s keys are Strings; each entry owns one. What
should the signature of the method that looks up an entry
in this table be? Here’s a first attempt:

```rust
impl<K, V> HashMap<K, V> where K: Eq + Hash
{
fn get(&self, key: K) -> Option<&V> { ... }
}
```

This makes sense: to look up an entry, you must provide a
key of the appropriate type for the table. But in this case, K
is String; this signature would force you to pass a String
by value to every call to get, which is clearly wasteful. You
really just need a reference to the key:

```rust
impl<K, V> HashMap<K, V> where K: Eq + Hash
{
  fn get(&self, key: &K) -> Option<&V> { ... }
}
```

This is `slightly better`, but now you have to pass the key as a
`&String`, so if you wanted to look up a constant string,
you’d have to write:

```hashtable.get(&"twenty-two".to_string())```

This is `ridiculous`: it allocates a String buﬀer on the heap
and copies the text into it, just so it can borrow it as a
&String, pass it to get, and then drop it.

if you can borrow an entry’s key as an `&Q`
and the resulting reference `hashes and compares` just the
way the key itself would, then clearly `&Q` ought to be an
acceptable key type. Since `String` implements
`Borrow<str>` and `Borrow<String>`, this `final version` of get
allows you to pass either `&String` or `&str` as a key, as
needed.

```rust
impl<K, V> HashMap<K, V> where K: Eq + Hash
{
  fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
  where 
    K: Borrow<Q>, 
    Q: Eq + Hash
  { 
    ... 
  }
}
```

## `From and Into`

the AsRef and AsMut traits borrow a reference of one type from another,
From and Into `take ownership` of their argument, transform
it, and then return ownership of the result back to the
caller.

You generally use Into to make your functions more
flexible in the arguments they accept. For example, if you
write:

```rust
use std::net::Ipv4Addr;
fn ping<A>(address: A) -> std::io::Result<bool>
where A: Into<Ipv4Addr>
{
  let ipv4_address = address.into();
  ...
}
```

then ping can accept not just an Ipv4Addr as an argument,
but also a u32 or a [u8; 4] array, since those types both
conveniently happen to implement `Into<Ipv4Addr>`.

we can make any of these calls:

```rust
println!("{:?}", ping(Ipv4Addr::new(23, 21, 68, 141))); // pass an Ipv4Addr
println!("{:?}", ping([66, 146, 219, 98])); // pass a [u8; 4]
println!("{:?}", ping(0xd076eb94_u32)); // pass a u32
```

```rust
type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
type GenericResult<T> = Result<T, GenericError>;

fn parse_i32_bytes(b: &[u8]) -> GenericResult<i32> {
  Ok(std::str::from_utf8(b)?.parse::<i32>()?)
}
```

Like most error types, Utf8Error and ParseIntError
implement the Error trait, and the standard library gives
us a blanket From impl for converting from anything that
implements `Error` to a `Box<dyn Error>`, which ? automatically uses:

```rust
impl<'a, E: Error + Send + Sync + 'a> From<E> for Box<dyn Error + Send + Sync + 'a> {
  fn from(err: E) -> Box<dyn Error + Send + Sync + 'a> {
    Box::new(err)
  }
}
```

## `TryFrom and TryInto`

```rust
pub trait TryFrom<T>: Sized {
type Error;
fn try_from(value: T) -> Result<Self, Self::Error>;
}
pub trait TryInto<T>: Sized {
type Error;
fn try_into(self) -> Result<T, Self::Error>;
}
```

The `try_into()` method gives us a Result, so we can
choose what to do in the exceptional case, such as a
number that’s too large to fit in the resulting type:

```rust
use std::convert::TryInto;
// Saturate on overflow, rather than wrapping
let smaller: i32 = huge.try_into().unwrap_or(i32::MAX);
```

If we want to also handle the negative case, we can use the
unwrap_or_else() method of Result:

```rust
let smaller: i32 = huge.try_into().unwrap_or_else(|_| {
  if huge >= 0 {
    i32::MAX
  } else {
    i32::MIN
  }
});
```

On the other hand, conversions between more
complex types might want to return more information:

```rust
impl TryInto<LinearShift> for Transform {
  type Error = TransformError;
  fn try_into(self) -> Result<LinearShift, Self::Error> {
    if !self.normalized() {
      return Err(TransformError::NotNormalized);
    }
    ...
  }
}
```

Where From and Into relate types with simple conversions,
TryFrom and TryInto, From and Into can be `used together`
to relate many types in a single crate.


## `ToOwned`

if you want to clone a `&str` or a `&[i32]`, you probably want is a
`String` or a `Vec<i32>`, but `Clone`’s definition doesn’t permit
that: by definition, cloning a `&T` must always return a value
of type `T`, and `str` and `[u8]` are unsized; they aren’t even
types that a function could return.  

The `std::borrow::ToOwned` trait provides a slightly looser
way to convert a reference to an owned value:

```rust
trait ToOwned {
  type Owned: Borrow<Self>;
  fn to_owned(&self) -> Self::Owned;
}
```

Unlike clone, which must return exactly Self, to_owned
can return anything you could borrow a &Self from: the
Owned type must implement Borrow<Self>. You can borrow
a &[T] from a Vec<T>, so [T] can implement
ToOwned<Owned=Vec<T>>, as long as T implements Clone,
so that we can copy the slice’s elements into the vector.

## `The Humble Cow`

in some cases
you cannot decide whether to borrow or own until the
program is running ; the std::borrow::Cow type (for
“clone on write”) provides one way to do this.
Its definition is shown here:

```rust
enum Cow<'a, B: ?Sized> where B: ToOwned
{
  Borrowed(&'a B),
  Owned(<B as ToOwned>::Owned),
}
```

A `Cow<B>` either borrows a shared reference to a B or owns
a value from which we could borrow such a reference.
Since Cow implements Deref, you can call methods on it as
if it were a shared reference to a B: if it’s Owned, it borrows
a shared reference to the owned value; and if it’s Borrowed,
it just hands out the reference it’s holding.

You can also get a mutable reference to a Cow’s value by
calling its `to_mut` method, which returns a &mut B. If the
Cow happens to be Cow::Borrowed, to_mut simply calls the
reference’s to_owned method to get its own copy of the
referent, changes the Cow into a Cow::Owned, and borrows a
mutable reference to the newly owned value. This is the
`“clone on write”` behavior the type’s name refers to.

Similarly, Cow has an `into_owned` method that promotes the
reference to an owned value, if necessary, and then returns
it, moving ownership to the caller and consuming the Cow
in the process.

`One common use` for Cow is to return either a statically
allocated string constant or a computed string. For
example, suppose you need to convert an error enum to a
message. Most of the variants can be handled with fixed
strings, but some of them have additional data that should
be included in the message. You can return a `Cow<'static, str>`:

```rust
use std::path::PathBuf;
use std::borrow::Cow;
fn describe(error: &Error) -> Cow<'static, str> {
  match *error {
    Error::OutOfMemory => "out of memory".into(),
    Error::StackOverflow => "stack overflow".into(),
    Error::MachineOnFire => "machine on fire".into(),
    Error::Unfathomable => "machine bewildered".into(),
    Error::FileNotFound(ref path) => {
      format!("file not found: {}", path.display()).into()
    }
  }
}
```

This code uses Cow’s implementation of Into to construct
the values. Most arms of this match statement return a
`Cow::Borrowed` referring to a statically allocated string.

But when we get a `FileNotFound` variant, we use format!`
to construct a message incorporating the given filename.
This arm of the match statement produces a `Cow::Owned` value.

`Callers` of describe that don’t need to change the value can
simply treat the Cow as a `&str`:

```rust
println!("Disaster has struck: {}", describe(&error));
```

Callers who do need an owned value can readily produce one:

```rust
let mut log: Vec<String> = Vec::new();
...
log.push(describe(&error).into_owned());
```

Using Cow helps describe and its callers `put oﬀ` allocation
until the moment it becomes necessary.
