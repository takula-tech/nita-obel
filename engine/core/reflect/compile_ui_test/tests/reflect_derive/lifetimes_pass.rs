//@check-pass
use obel_reflect::Reflect;

// Reason: Reflection relies on `Any` which requires `'static`
#[derive(Reflect)]
struct Foo<'a: 'static> {
    #[reflect(ignore)]
    value: &'a str,
}
