//! Contains code related specifically to Obel's type registration.

use quote::quote;
use syn::Type;

use crate::{
    derive_data::ReflectMeta, serialization::SerializationDataDef,
    where_clause_options::WhereClauseOptions,
};

/// Creates the `GetTypeRegistration` impl for the given type data.
pub(crate) fn impl_get_type_registration<'a>(
    meta: &ReflectMeta,
    where_clause_options: &WhereClauseOptions,
    serialization_data: Option<&SerializationDataDef>,
    type_dependencies: Option<impl Iterator<Item = &'a Type>>,
) -> proc_macro2::TokenStream {
    let type_path = meta.type_path();
    let obel_reflect_path = meta.obel_reflect_path();
    let registration_data = meta.attrs().idents();

    let type_deps_fn = type_dependencies.map(|deps| {
        quote! {
            #[inline(never)]
            fn register_type_dependencies(registry: &mut #obel_reflect_path::TypeRegistry) {
                #(<#deps as #obel_reflect_path::__macro_exports::RegisterForReflection>::__register(registry);)*
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = type_path.generics().split_for_impl();
    let where_reflect_clause = where_clause_options.extend_where_clause(where_clause);

    let from_reflect_data = if meta.from_reflect().should_auto_derive() {
        Some(quote! {
            registration.insert::<#obel_reflect_path::ReflectFromReflect>(#obel_reflect_path::FromType::<Self>::from_type());
        })
    } else {
        None
    };

    let serialization_data = serialization_data.map(|data| {
        let serialization_data = data.as_serialization_data(obel_reflect_path);
        quote! {
            registration.insert::<#obel_reflect_path::serde::SerializationData>(#serialization_data);
        }
    });

    quote! {
        #[allow(unused_mut)]
        impl #impl_generics #obel_reflect_path::GetTypeRegistration for #type_path #ty_generics #where_reflect_clause {
            fn get_type_registration() -> #obel_reflect_path::TypeRegistration {
                let mut registration = #obel_reflect_path::TypeRegistration::of::<Self>();
                registration.insert::<#obel_reflect_path::ReflectFromPtr>(#obel_reflect_path::FromType::<Self>::from_type());
                #from_reflect_data
                #serialization_data
                #(registration.insert::<#registration_data>(#obel_reflect_path::FromType::<Self>::from_type());)*
                registration
            }

            #type_deps_fn
        }
    }
}
