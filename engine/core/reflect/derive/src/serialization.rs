use std::collections::HashMap;

use quote::quote;
use syn::{Path, spanned::Spanned};

use crate::{
    attr::field::{DefaultBehavior, ReflectIgnoreBehavior},
    derive_data::StructField,
};
use obel_reflect_utils::FQDefault;

type ReflectionIndex = usize;

/// Collected serialization data used to generate a `SerializationData` type.
pub(crate) struct SerializationDataDef {
    /// Maps a field's _reflection_ index to its [`SkippedFieldDef`] if marked as `#[reflect(skip_serializing)]`.
    skipped: HashMap<ReflectionIndex, SkippedFieldDef>,
}

impl SerializationDataDef {
    /// Attempts to create a new `SerializationDataDef` from the given collection of fields.
    ///
    /// Returns `Ok(Some(data))` if there are any fields needing to be skipped during serialization.
    /// Otherwise, returns `Ok(None)`.
    pub fn new(
        fields: &[StructField<'_>],
        obel_reflect_path: &Path,
    ) -> Result<Option<Self>, syn::Error> {
        let mut skipped = <HashMap<_, _>>::default();

        for field in fields {
            match field.attrs.ignore {
                ReflectIgnoreBehavior::IgnoreSerialization => {
                    skipped.insert(
                        field.reflection_index.ok_or_else(|| {
                            syn::Error::new(
                                field.data.span(),
                                "internal error: field is missing a reflection index",
                            )
                        })?,
                        SkippedFieldDef::new(field, obel_reflect_path)?,
                    );
                }
                _ => continue,
            }
        }

        if skipped.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self {
                skipped,
            }))
        }
    }

    /// Returns a `TokenStream` containing an initialized `SerializationData` type.
    pub fn as_serialization_data(&self, obel_reflect_path: &Path) -> proc_macro2::TokenStream {
        let fields = self.skipped.iter().map(
            |(
                reflection_index,
                SkippedFieldDef {
                    default_fn,
                },
            )| {
                quote! {(
                    #reflection_index,
                    #obel_reflect_path::serde::SkippedField::new(#default_fn)
                )}
            },
        );
        quote! {
            #obel_reflect_path::serde::SerializationData::new(
                ::core::iter::IntoIterator::into_iter([#(#fields),*])
            )
        }
    }
}

/// Collected field data used to generate a `SkippedField` type.
pub(crate) struct SkippedFieldDef {
    /// The default function for this field.
    ///
    /// This is of type `fn() -> Box<dyn Reflect>`.
    default_fn: proc_macro2::TokenStream,
}

impl SkippedFieldDef {
    pub fn new(field: &StructField<'_>, obel_reflect_path: &Path) -> Result<Self, syn::Error> {
        let ty = &field.data.ty;

        let default_fn = match &field.attrs.default {
            DefaultBehavior::Func(func) => quote! {
              || { #obel_reflect_path::__macro_exports::alloc_utils::Box::new(#func()) }
            },
            _ => quote! {
              || { #obel_reflect_path::__macro_exports::alloc_utils::Box::new(<#ty as #FQDefault>::default()) }
            },
        };

        Ok(Self {
            default_fn,
        })
    }
}
