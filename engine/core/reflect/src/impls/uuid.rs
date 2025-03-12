#[warn(unused_imports)]
use crate::{ReflectDeserialize, ReflectSerialize, std_traits::ReflectDefault};
use obel_reflect_derive::impl_reflect_opaque;

impl_reflect_opaque!(::uuid::Uuid(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Hash));
