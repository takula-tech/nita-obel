/// Which trait the macro explicitly implements.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ReflectTraitToImpl {
    Reflect,
    FromReflect,
    TypePath,
}
