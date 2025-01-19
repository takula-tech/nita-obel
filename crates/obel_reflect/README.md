# obel App

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![Crates.io](https://img.shields.io/crates/v/obel_reflect.svg)](https://crates.io/crates/obel_reflect)
[![Downloads](https://img.shields.io/crates/d/obel_reflect.svg)](https://crates.io/crates/obel_reflect)
[![Docs](https://docs.rs/obel_reflect/badge.svg)](https://docs.rs/obel_reflect/latest/obel_reflect/)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/obel)

Reflection in Rust.

[Reflection] is a powerful tool provided within many programming languages
that allows for meta-programming: using information _about_ the program to
_affect_ the program.
In other words, reflection allows us to inspect the program itself, its
syntax, and its type information at runtime.

This crate adds this missing reflection functionality to Rust.
Though it was made with the [Bevy] game engine in mind,
it's a general-purpose solution that can be used in any Rust project.

At a very high level, this crate allows you to:

- Dynamically interact with Rust values
- Access type metadata at runtime
- Serialize and deserialize (i.e. save and load) data

It's important to note that because of missing features in Rust,
there are some [limitations] with this crate.

# The `Reflect` and `PartialReflect` traits

At the root of [`bevy_reflect`] is the [`PartialReflect`] trait.

Its purpose is to allow dynamic [introspection] of values,
following Rust's type system through a system of [subtraits].

Its primary purpose is to allow all implementors to be passed around
as a `dyn PartialReflect` trait object in one of the following forms:

- `&dyn PartialReflect`
- `&mut dyn PartialReflect`
- `Box<dyn PartialReflect>`

This allows values of types implementing `PartialReflect`
to be operated upon completely dynamically (at a small [runtime cost]).

Building on `PartialReflect` is the [`Reflect`] trait.

`PartialReflect` is a supertrait of `Reflect`
so any type implementing `Reflect` implements `PartialReflect` by definition.
`dyn Reflect` trait objects can be used similarly to `dyn PartialReflect`,
but `Reflect` is also often used in trait bounds (like `T: Reflect`).

The distinction between `PartialReflect` and `Reflect` is summarized in the following:

- `PartialReflect` is a trait for interacting with values under `bevy_reflect`'s data model.
  This means values implementing `PartialReflect` can be dynamically constructed and introspected.
- The `Reflect` trait, however, ensures that the interface exposed by `PartialReflect`
  on types which additionally implement `Reflect` mirrors the structure of a single Rust type.
- This means `dyn Reflect` trait objects can be directly downcasted to concrete types,
  where `dyn PartialReflect` trait object cannot.
- `Reflect`, since it provides a stronger type-correctness guarantee,
  is the trait used to interact with [the type registry].

## Converting between `PartialReflect` and `Reflect`

Since `T: Reflect` implies `T: PartialReflect`, conversion from a `dyn Reflect` to a `dyn PartialReflect`
trait object (upcasting) is infallible and can be performed with one of the following methods.
Note that these are temporary while [the language feature for dyn upcasting coercion] is experimental:

- [`PartialReflect::as_partial_reflect`] for `&dyn PartialReflect`
- [`PartialReflect::as_partial_reflect_mut`] for `&mut dyn PartialReflect`
- [`PartialReflect::into_partial_reflect`] for `Box<dyn PartialReflect>`

For conversion in the other direction — downcasting `dyn PartialReflect` to `dyn Reflect` —
there are fallible methods:

- [`PartialReflect::try_as_reflect`] for `&dyn Reflect`
- [`PartialReflect::try_as_reflect_mut`] for `&mut dyn Reflect`
- [`PartialReflect::try_into_reflect`] for `Box<dyn Reflect>`

Additionally, [`FromReflect::from_reflect`] can be used to convert a `dyn PartialReflect` to a concrete type
which implements `Reflect`.

# Implementing `Reflect`

Implementing `Reflect` (and `PartialReflect`) is easily done using the provided [derive macro]:

```
# use bevy_reflect::Reflect;
#[derive(Reflect)]
struct MyStruct {
  foo: i32
}
```

This will automatically generate the implementation of `Reflect` for any struct or enum.

It will also generate other very important trait implementations used for reflection:

- [`GetTypeRegistration`]
- [`Typed`]
- [`Struct`], [`TupleStruct`], or [`Enum`] depending on the type

## Requirements

We can implement `Reflect` on any type that satisfies _both_ of the following conditions:

- The type implements `Any`, `Send`, and `Sync`.
  For the `Any` requirement to be satisfied, the type itself must have a [`'static` lifetime].
- All fields and sub-elements themselves implement `Reflect`
  (see the [derive macro documentation] for details on how to ignore certain fields when deriving).

Additionally, using the derive macro on enums requires a third condition to be met:

- All fields and sub-elements must implement [`FromReflect`]—
  another important reflection trait discussed in a later section.

# The Reflection Subtraits

Since [`PartialReflect`] is meant to cover any and every type, this crate also comes with a few
more traits to accompany `PartialReflect` and provide more specific interactions.
We refer to these traits as the _reflection subtraits_ since they all have `PartialReflect` as a supertrait.
The current list of reflection subtraits include:

- [`Tuple`]
- [`Array`]
- [`List`]
- [`Set`]
- [`Map`]
- [`Struct`]
- [`TupleStruct`]
- [`Enum`]
- [`Function`] (requires the `functions` feature)

As mentioned previously, the last three are automatically implemented by the [derive macro].

Each of these traits come with their own methods specific to their respective category.
For example, we can access our struct's fields by name using the [`Struct::field`] method.

```
# use bevy_reflect::{PartialReflect, Reflect, Struct};
# #[derive(Reflect)]
# struct MyStruct {
#   foo: i32
# }
let my_struct: Box<dyn Struct> = Box::new(MyStruct {
  foo: 123
});
let foo: &dyn PartialReflect = my_struct.field("foo").unwrap();
assert_eq!(Some(&123), foo.try_downcast_ref::<i32>());
```

Since most data is passed around as `dyn PartialReflect` or `dyn Reflect` trait objects,
the `PartialReflect` trait has methods for going to and from these subtraits.

[`PartialReflect::reflect_kind`], [`PartialReflect::reflect_ref`],
[`PartialReflect::reflect_mut`], and [`PartialReflect::reflect_owned`] all return
an enum that respectively contains zero-sized, immutable, mutable, and owned access to the type as a subtrait object.

For example, we can get out a `dyn Tuple` from our reflected tuple type using one of these methods.

```
# use bevy_reflect::{PartialReflect, ReflectRef};
let my_tuple: Box<dyn PartialReflect> = Box::new((1, 2, 3));
let my_tuple = my_tuple.reflect_ref().as_tuple().unwrap();
assert_eq!(3, my_tuple.field_len());
```

And to go back to a general-purpose `dyn PartialReflect`,
we can just use the matching [`PartialReflect::as_partial_reflect`], [`PartialReflect::as_partial_reflect_mut`],
or [`PartialReflect::into_partial_reflect`] methods.

## Opaque Types

Some types don't fall under a particular subtrait.

These types hide their internal structure to reflection,
either because it is not possible, difficult, or not useful to reflect its internals.
Such types are known as _opaque_ types.

This includes truly opaque types like `String` or `Instant`,
but also includes all the primitive types (e.g. `bool`, `usize`, etc.)
since they can't be broken down any further.

# Dynamic Types

Each subtrait comes with a corresponding _dynamic_ type.

The available dynamic types are:

- [`DynamicTuple`]
- [`DynamicArray`]
- [`DynamicList`]
- [`DynamicMap`]
- [`DynamicStruct`]
- [`DynamicTupleStruct`]
- [`DynamicEnum`]

These dynamic types may contain any arbitrary reflected data.

```
# use bevy_reflect::{DynamicStruct, Struct};
let mut data = DynamicStruct::default();
data.insert("foo", 123_i32);
assert_eq!(Some(&123), data.field("foo").unwrap().try_downcast_ref::<i32>())
```

They are most commonly used as "proxies" for other types,
where they contain the same data as— and therefore, represent— a concrete type.
The [`PartialReflect::clone_value`] method will return a dynamic type for all non-opaque types,
allowing all types to essentially be "cloned".
And since dynamic types themselves implement [`PartialReflect`],
we may pass them around just like most other reflected types.

```
# use bevy_reflect::{DynamicStruct, PartialReflect, Reflect};
# #[derive(Reflect)]
# struct MyStruct {
#   foo: i32
# }
let original: Box<dyn Reflect> = Box::new(MyStruct {
  foo: 123
});

// `cloned` will be a `DynamicStruct` representing a `MyStruct`
let cloned: Box<dyn PartialReflect> = original.clone_value();
assert!(cloned.represents::<MyStruct>());
```

## Patching

These dynamic types come in handy when needing to apply multiple changes to another type.
This is known as "patching" and is done using the [`PartialReflect::apply`] and [`PartialReflect::try_apply`] methods.

```
# use bevy_reflect::{DynamicEnum, PartialReflect};
let mut value = Some(123_i32);
let patch = DynamicEnum::new("None", ());
value.apply(&patch);
assert_eq!(None, value);
```

## `FromReflect`

It's important to remember that dynamic types are _not_ the concrete type they may be representing.
A common mistake is to treat them like such when trying to cast back to the original type
or when trying to make use of a reflected trait which expects the actual type.

```should_panic
# use bevy_reflect::{DynamicStruct, PartialReflect, Reflect};
# #[derive(Reflect)]
# struct MyStruct {
#   foo: i32
# }
let original: Box<dyn Reflect> = Box::new(MyStruct {
  foo: 123
});

let cloned: Box<dyn PartialReflect> = original.clone_value();
let value = cloned.try_take::<MyStruct>().unwrap(); // PANIC!
```

To resolve this issue, we'll need to convert the dynamic type to the concrete one.
This is where [`FromReflect`] comes in.

`FromReflect` is a trait that allows an instance of a type to be generated from a
dynamic representation— even partial ones.
And since the [`FromReflect::from_reflect`] method takes the data by reference,
this can be used to effectively clone data (to an extent).

It is automatically implemented when [deriving `Reflect`] on a type unless opted out of
using `#[reflect(from_reflect = false)]` on the item.

```
# use bevy_reflect::{FromReflect, PartialReflect, Reflect};
#[derive(Reflect)]
struct MyStruct {
  foo: i32
}
let original: Box<dyn Reflect> = Box::new(MyStruct {
  foo: 123
});

let cloned: Box<dyn PartialReflect> = original.clone_value();
let value = <MyStruct as FromReflect>::from_reflect(&*cloned).unwrap(); // OK!
```

When deriving, all active fields and sub-elements must also implement `FromReflect`.

Fields can be given default values for when a field is missing in the passed value or even ignored.
Ignored fields must either implement [`Default`] or have a default function specified
using `#[reflect(default = "path::to::function")]`.

See the [derive macro documentation](derive@crate::FromReflect) for details.

All primitives and simple types implement `FromReflect` by relying on their [`Default`] implementation.

# Path navigation

The [`GetPath`] trait allows accessing arbitrary nested fields of an [`PartialReflect`] type.

Using `GetPath`, it is possible to use a path string to access a specific field
of a reflected type.

```
# use bevy_reflect::{Reflect, GetPath};
#[derive(Reflect)]
struct MyStruct {
  value: Vec<Option<u32>>
}

let my_struct = MyStruct {
  value: vec![None, None, Some(123)],
};
assert_eq!(
  my_struct.path::<u32>(".value[2].0").unwrap(),
  &123,
);
```

# Type Registration

This crate also comes with a [`TypeRegistry`] that can be used to store and retrieve additional type metadata at runtime,
such as helper types and trait implementations.

The [derive macro] for [`Reflect`] also generates an implementation of the [`GetTypeRegistration`] trait,
which is used by the registry to generate a [`TypeRegistration`] struct for that type.
We can then register additional [type data] we want associated with that type.

For example, we can register [`ReflectDefault`] on our type so that its `Default` implementation
may be used dynamically.

```
# use bevy_reflect::{Reflect, TypeRegistry, prelude::ReflectDefault};
#[derive(Reflect, Default)]
struct MyStruct {
  foo: i32
}
let mut registry = TypeRegistry::empty();
registry.register::<MyStruct>();
registry.register_type_data::<MyStruct, ReflectDefault>();

let registration = registry.get(core::any::TypeId::of::<MyStruct>()).unwrap();
let reflect_default = registration.data::<ReflectDefault>().unwrap();

let new_value: Box<dyn Reflect> = reflect_default.default();
assert!(new_value.is::<MyStruct>());
```

Because this operation is so common, the derive macro actually has a shorthand for it.
By using the `#[reflect(Trait)]` attribute, the derive macro will automatically register a matching,
in-scope `ReflectTrait` type within the `GetTypeRegistration` implementation.

```
use bevy_reflect::prelude::{Reflect, ReflectDefault};

#[derive(Reflect, Default)]
#[reflect(Default)]
struct MyStruct {
  foo: i32
}
```

## Reflecting Traits

Type data doesn't have to be tied to a trait, but it's often extremely useful to create trait type data.
These allow traits to be used directly on a `dyn Reflect` (and not a `dyn PartialReflect`)
while utilizing the underlying type's implementation.

For any [object-safe] trait, we can easily generate a corresponding `ReflectTrait` type for our trait
using the [`#[reflect_trait]`](reflect_trait) macro.

```
# use bevy_reflect::{Reflect, reflect_trait, TypeRegistry};
#[reflect_trait] // Generates a `ReflectMyTrait` type
pub trait MyTrait {}
impl<T: Reflect> MyTrait for T {}

let mut registry = TypeRegistry::new();
registry.register_type_data::<i32, ReflectMyTrait>();
```

The generated type data can be used to convert a valid `dyn Reflect` into a `dyn MyTrait`.
See the [dynamic types example](https://github.com/bevyengine/bevy/blob/latest/examples/reflection/dynamic_types.rs)
for more information and usage details.

# Serialization

By using reflection, we are also able to get serialization capabilities for free.
In fact, using [`bevy_reflect`] can result in faster compile times and reduced code generation over
directly deriving the [`serde`] traits.

The way it works is by moving the serialization logic into common serializers and deserializers:

- [`ReflectSerializer`]
- [`TypedReflectSerializer`]
- [`ReflectDeserializer`]
- [`TypedReflectDeserializer`]

All of these structs require a reference to the [registry] so that [type information] can be retrieved,
as well as registered type data, such as [`ReflectSerialize`] and [`ReflectDeserialize`].

The general entry point are the "untyped" versions of these structs.
These will automatically extract the type information and pass them into their respective "typed" version.

The output of the `ReflectSerializer` will be a map, where the key is the [type path]
and the value is the serialized data.
The `TypedReflectSerializer` will simply output the serialized data.

The `ReflectDeserializer` can be used to deserialize this map and return a `Box<dyn Reflect>`,
where the underlying type will be a dynamic type representing some concrete type (except for opaque types).

Again, it's important to remember that dynamic types may need to be converted to their concrete counterparts
in order to be used in certain cases.
This can be achieved using [`FromReflect`].

```
# use serde::de::DeserializeSeed;
# use bevy_reflect::{
#     serde::{ReflectSerializer, ReflectDeserializer},
#     Reflect, PartialReflect, FromReflect, TypeRegistry
# };
#[derive(Reflect, PartialEq, Debug)]
struct MyStruct {
  foo: i32
}

let original_value = MyStruct {
  foo: 123
};

// Register
let mut registry = TypeRegistry::new();
registry.register::<MyStruct>();

// Serialize
let reflect_serializer = ReflectSerializer::new(original_value.as_partial_reflect(), &registry);
let serialized_value: String = ron::to_string(&reflect_serializer).unwrap();

// Deserialize
let reflect_deserializer = ReflectDeserializer::new(&registry);
let deserialized_value: Box<dyn PartialReflect> = reflect_deserializer.deserialize(
  &mut ron::Deserializer::from_str(&serialized_value).unwrap()
).unwrap();

// Convert
let converted_value = <MyStruct as FromReflect>::from_reflect(&*deserialized_value).unwrap();

assert_eq!(original_value, converted_value);
```

# Limitations

While this crate offers a lot in terms of adding reflection to Rust,
it does come with some limitations that don't make it as featureful as reflection
in other programming languages.

## Non-Static Lifetimes

One of the most obvious limitations is the `'static` requirement.
Rust requires fields to define a lifetime for referenced data,
but [`Reflect`] requires all types to have a `'static` lifetime.
This makes it impossible to reflect any type with non-static borrowed data.

## Generic Function Reflection

Another limitation is the inability to reflect over generic functions directly. It can be done, but will
typically require manual monomorphization (i.e. manually specifying the types the generic method can
take).

## Manual Registration

Since Rust doesn't provide built-in support for running initialization code before `main`,
there is no way for `bevy_reflect` to automatically register types into the [type registry].
This means types must manually be registered, including their desired monomorphized
representations if generic.

# Features

## `obel`

| Default |             Dependencies              |
| :-----: | :-----------------------------------: |
|   ❌    | [`bevy_math`], [`glam`], [`smallvec`] |

This feature makes it so that the appropriate reflection traits are implemented on all the types
necessary for the [Obel] game engine.
enables the optional dependencies: [`bevy_math`], [`glam`], and [`smallvec`].
These dependencies are used by the [Bevy] game engine and must define their reflection implementations
within this crate due to Rust's [orphan rule].

## `functions`

| Default |           Dependencies            |
| :-----: | :-------------------------------: |
|   ❌    | [`bevy_reflect_derive/functions`] |

This feature allows creating a [`DynamicFunction`] or [`DynamicFunctionMut`] from Rust functions. Dynamic
functions can then be called with valid [`ArgList`]s.

For more information, read the [`func`] module docs.

## `documentation`

| Default |             Dependencies              |
| :-----: | :-----------------------------------: |
|   ❌    | [`bevy_reflect_derive/documentation`] |

This feature enables capturing doc comments as strings for items that [derive `Reflect`].
Documentation information can then be accessed at runtime on the [`TypeInfo`] of that item.

This can be useful for generating documentation for scripting language interop or
for displaying tooltips in an editor.

## `debug`

| Default | Dependencies  |
| :-----: | :-----------: |
|   ✅    | `debug_stack` |

This feature enables useful debug features for reflection.

This includes the `debug_stack` feature,
which enables capturing the type stack when serializing or deserializing a type
and displaying it in error messages.

[Reflection]: https://en.wikipedia.org/wiki/Reflective_programming
[Bevy]: https://bevyengine.org/
[limitations]: #limitations
[`bevy_reflect`]: crate
[introspection]: https://en.wikipedia.org/wiki/Type_introspection
[subtraits]: #the-reflection-subtraits
[the type registry]: #type-registration
[runtime cost]: https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch
[the language feature for dyn upcasting coercion]: https://github.com/rust-lang/rust/issues/65991
[derive macro]: derive@crate::Reflect
[`'static` lifetime]: https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound
[`Function`]: crate::func::Function
[derive macro documentation]: derive@crate::Reflect
[deriving `Reflect`]: derive@crate::Reflect
[type data]: TypeData
[`ReflectDefault`]: std_traits::ReflectDefault
[object-safe]: https://doc.rust-lang.org/reference/items/traits.html#object-safety
[`serde`]: ::serde
[`ReflectSerializer`]: serde::ReflectSerializer
[`TypedReflectSerializer`]: serde::TypedReflectSerializer
[`ReflectDeserializer`]: serde::ReflectDeserializer
[`TypedReflectDeserializer`]: serde::TypedReflectDeserializer
[registry]: TypeRegistry
[type information]: TypeInfo
[type path]: TypePath
[type registry]: TypeRegistry
[`bevy_math`]: https://docs.rs/bevy_math/latest/bevy_math/
[`glam`]: https://docs.rs/glam/latest/glam/
[`smallvec`]: https://docs.rs/smallvec/latest/smallvec/
[orphan rule]: https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type:~:text=But%20we%20can%E2%80%99t,implementation%20to%20use.
[`bevy_reflect_derive/documentation`]: bevy_reflect_derive
[`bevy_reflect_derive/functions`]: bevy_reflect_derive
[`DynamicFunction`]: crate::func::DynamicFunction
[`DynamicFunctionMut`]: crate::func::DynamicFunctionMut
[`ArgList`]: crate::func::ArgList
[derive `Reflect`]: derive@crate::Re
