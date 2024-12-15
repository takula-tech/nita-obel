# Obel Reflect

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![Crates.io](https://img.shields.io/crates/v/obel.svg)](https://crates.io/crates/obel_reflect)
[![Downloads](https://img.shields.io/crates/d/obel_reflect.svg)](https://crates.io/crates/obel_reflect)
[![Docs](https://docs.rs/obel_reflect/badge.svg)](https://docs.rs/obel_reflect/latest/obel_reflect/)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/obel)

This crate enables you to dynamically interact with Rust types:

- Derive the `Reflect` traits
- Interact with fields using their names (for named structs) or indices (for tuple structs)
- "Patch" your types with new values
- Look up nested fields using "path strings"
- Iterate over struct fields
- Automatically serialize and deserialize via Serde (without explicit serde impls)
- Trait "reflection"

## Features

### Derive the `Reflect` traits

```rust ignore
// this will automatically implement the `Reflect` trait and the `Struct` trait (because the type is a struct)
#[derive(Reflect)]
struct Foo {
    a: u32,
    b: Bar,
    c: Vec<i32>,
    d: Vec<Baz>,
}

// this will automatically implement the `Reflect` trait and the `TupleStruct` trait (because the type is a tuple struct)
#[derive(Reflect)]
struct Bar(String);

#[derive(Reflect)]
struct Baz {
    value: f32,
}

// We will use this value to illustrate `obel_reflect` features
let mut foo = Foo {
    a: 1,
    b: Bar("hello".to_string()),
    c: vec![1, 2],
    d: vec![Baz { value: 3.14 }],
};
```

### Interact with fields using their names

```rust ignore
assert_eq!(*foo.get_field::<u32>("a").unwrap(), 1);

*foo.get_field_mut::<u32>("a").unwrap() = 2;

assert_eq!(foo.a, 2);
```

### "Patch" your types with new values

```rust ignore
let mut dynamic_struct = DynamicStruct::default();
dynamic_struct.insert("a", 42u32);
dynamic_struct.insert("c", vec![3, 4, 5]);

foo.apply(&dynamic_struct);

assert_eq!(foo.a, 42);
assert_eq!(foo.c, vec![3, 4, 5]);
```

### Look up nested fields using "path strings"

```rust ignore
let value = *foo.get_path::<f32>("d[0].value").unwrap();
assert_eq!(value, 3.14);
```

### Iterate over struct fields

```rust ignore
for (i, value: &Reflect) in foo.iter_fields().enumerate() {
    let field_name = foo.name_at(i).unwrap();
    if let Some(value) = value.downcast_ref::<u32>() {
        println!("{} is a u32 with the value: {}", field_name, *value);
    }
}
```

### Automatically serialize and deserialize via Serde (without explicit serde impls)

```rust ignore
let mut registry = TypeRegistry::default();
registry.register::<u32>();
registry.register::<i32>();
registry.register::<f32>();
registry.register::<String>();
registry.register::<Bar>();
registry.register::<Baz>();

let serializer = ReflectSerializer::new(&foo, &registry);
let serialized = ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default()).unwrap();

let mut deserializer = ron::de::Deserializer::from_str(&serialized).unwrap();
let reflect_deserializer = ReflectDeserializer::new(&registry);
let value = reflect_deserializer.deserialize(&mut deserializer).unwrap();
let dynamic_struct = value.take::<DynamicStruct>().unwrap();

assert!(foo.reflect_partial_eq(&dynamic_struct).unwrap());
```

### Trait "reflection"

Call a trait on a given `&dyn Reflect` reference without knowing the underlying type!

```rust ignore
#[derive(Reflect)]
#[reflect(DoThing)]
struct MyType {
    value: String,
}

impl DoThing for MyType {
    fn do_thing(&self) -> String {
        format!("{} World!", self.value)
    }
}

#[reflect_trait]
pub trait DoThing {
    fn do_thing(&self) -> String;
}

// First, lets box our type as a Box<dyn Reflect>
let reflect_value: Box<dyn Reflect> = Box::new(MyType {
    value: "Hello".to_string(),
});

// This means we no longer have direct access to MyType or its methods. We can only call Reflect methods on reflect_value.
// What if we want to call `do_thing` on our type? We could downcast using reflect_value.downcast_ref::<MyType>(), but what if we
// don't know the type at compile time?

// Normally in rust we would be out of luck at this point. Lets use our new reflection powers to do something cool!
let mut type_registry = TypeRegistry::default();
type_registry.register::<MyType>();

// The #[reflect] attribute we put on our DoThing trait generated a new `ReflectDoThing` struct, which implements TypeData.
// This was added to MyType's TypeRegistration.
let reflect_do_thing = type_registry
    .get_type_data::<ReflectDoThing>(reflect_value.type_id())
    .unwrap();

// We can use this generated type to convert our `&dyn Reflect` reference to a `&dyn DoThing` reference
let my_trait: &dyn DoThing = reflect_do_thing.get(&*reflect_value).unwrap();

// Which means we can now call do_thing(). Magic!
println!("{}", my_trait.do_thing());

// This works because the #[reflect(MyTrait)] we put on MyType informed the Reflect derive to insert a new instance
// of ReflectDoThing into MyType's registration. The instance knows how to cast &dyn Reflect to &dyn DoThing, because it
// knows that &dyn Reflect should first be downcasted to &MyType, which can then be safely casted to &dyn DoThing
```

## Why make this?

The whole point of Rust is static safety! Why build something that makes it easy to throw it all away?

- Some problems are inherently dynamic (scripting, some types of serialization / deserialization)
- Sometimes the dynamic way is easier
- Sometimes the dynamic way puts less burden on your users to derive a bunch of traits (this was a big motivator for the obel project)

## `Sequence Diagram`

The following sequence diagram illustrates how a type with `#[derive(Reflect)]` is processed:

```mermaid
sequenceDiagram
    box src/
    participant User Code
    participant TypeRegistry
    end

    box derive/src/lib.rs
    participant derive_reflect
    end

    box derive/src/derive_data.rs
    participant ReflectDerive
    participant ReflectMeta
    end

    box derive/src/container_attributes.rs
    participant ContainerAttributes
    participant TraitImpl
    end

    box derive/src/impls/
    participant Impls
    participant StructImpl
    participant EnumImpl
    participant TupleStructImpl
    end

    box derive/src/from_reflect.rs
    participant FromReflect
    end

    User Code->>derive_reflect: #[derive(Reflect)]
    Note over derive_reflect: Entry point for derive macro processing

    derive_reflect->>derive_reflect: parse_macro_input!(input as DeriveInput)
    Note over derive_reflect: Converts raw TokenStream into AST

    derive_reflect->>ReflectDerive: from_input(ast, ReflectTraitToImpl::Reflect)

    ReflectDerive->>ReflectMeta: new()
    Note over ReflectMeta: Creates metadata containing:<br/>1. Type path & name<br/>2. Generics info<br/>3. Documentation<br/>4. Custom attributes

    ReflectDerive->>ReflectDerive: Determine type kind
    Note over ReflectDerive: Analyzes if input is:<br/>1. Struct<br/>2. TupleStruct<br/>3. Enum<br/>4. Opaque<br/>Creates corresponding ReflectData variant

    ReflectDerive->>ContainerAttributes: parse_meta_list()
    Note over ContainerAttributes: Parses #[reflect(...)] attributes

    ContainerAttributes->>TraitImpl: Process special traits
    Note over TraitImpl: Handles special traits with custom implementations:<br/>1. Debug - Uses type's Debug impl<br/>2. Hash - Uses type's Hash impl<br/>3. PartialEq - Uses type's PartialEq impl<br/>4. Default - Enables optimized FromReflect

    ContainerAttributes->>ContainerAttributes: Process standard traits
    Note over ContainerAttributes: For each trait:<br/>1. Validate trait name<br/>2. Create ReflectTrait ident<br/>3. Check for duplicates<br/>4. Register trait implementations

    ContainerAttributes->>ContainerAttributes: Configure FromReflect
    Note over ContainerAttributes: 1. Check auto_derive settings<br/>2. Parse from_reflect attributes<br/>3. Configure field-level behaviors

    ContainerAttributes-->>ReflectDerive: Complete configuration

    ReflectDerive->>Impls: Generate implementations

    alt Type is Struct
        Impls->>StructImpl: impl_struct()
        Note over StructImpl: Generates:<br/>1. Struct trait impl<br/>2. Field accessors<br/>3. Dynamic struct conversion
    else Type is Enum
        Impls->>EnumImpl: impl_enum()
        Note over EnumImpl: Generates:<br/>1. Enum trait impl<br/>2. Variant handling<br/>3. Dynamic enum conversion
    else Type is TupleStruct
        Impls->>TupleStructImpl: impl_tuple_struct()
        Note over TupleStructImpl: Generates:<br/>1. TupleStruct trait impl<br/>2. Index-based access<br/>3. Dynamic tuple conversion
    end

    Impls->>Impls: Generate Reflect impl
    Note over Impls: Implements core reflection methods:<br/>1. type_name() - Get type name<br/>2. get_type_info() - Get static type info<br/>3. as_any() - Cast to Any<br/>4. as_reflect() - Get Reflect trait object<br/>5. apply() - Update from another Reflect<br/>6. set() - Set value from Reflect

    Impls->>Impls: Generate GetTypeRegistration
    Note over Impls: Creates type registration with:<br/>1. Type info - Static type metadata<br/>2. Trait implementations - Special trait handlers<br/>3. Type data - Additional type information<br/>4. Custom attributes - User-defined metadata

    alt FromReflect Auto-derive enabled
        ReflectDerive->>FromReflect: Generate FromReflect impl
        Note over FromReflect: Implements from_reflect method with:<br/>1. Type validation - Ensures correct type<br/>2. Field conversion - Converts each field<br/>3. Error handling - Handles invalid data<br/>4. Default handling - Uses Default if configured
        FromReflect-->>ReflectDerive: FromReflect implementation
    end

    alt Special Traits Registered
        Impls->>Impls: Generate special trait impls
        Note over Impls: Implements registered traits:<br/>1. reflect_hash() - Custom hash implementation<br/>2. reflect_partial_eq() - Custom equality check<br/>3. debug() - Custom debug formatting<br/>4. Default handling - Optional default construction
    end

    Impls-->>ReflectDerive: All implementations

    ReflectDerive->>ReflectDerive: Combine implementations
    Note over ReflectDerive: 1. Merges all trait implementations<br/>2. Validates compatibility<br/>3. Generates final TokenStream

    ReflectDerive-->>derive_reflect: Final TokenStream
    derive_reflect-->>User Code: Expanded macro code

    Note over User Code: Runtime Type Registration
    User Code->>TypeRegistry: register::<Type>()
    Note over TypeRegistry: Registers type for runtime reflection:<br/>1. Stores type information<br/>2. Enables dynamic access<br/>3. Powers serialization/deserialization
```
