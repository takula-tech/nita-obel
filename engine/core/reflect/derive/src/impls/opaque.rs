use crate::{
    ReflectMeta,
    impls::{common_partial_reflect_methods, impl_full_reflect, impl_type_path, impl_typed},
    where_clause_options::WhereClauseOptions,
};
use obel_reflect_utils::{FQClone, FQOption, FQResult};
use quote::quote;

/// Implements `GetTypeRegistration` and `Reflect` for the given type data.
pub(crate) fn impl_opaque(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    let obel_reflect_path = meta.obel_reflect_path();
    let type_path = meta.type_path();

    #[cfg(feature = "documentation")]
    let with_docs = {
        let doc = quote::ToTokens::to_token_stream(meta.doc());
        Some(quote!(.with_docs(#doc)))
    };
    #[cfg(not(feature = "documentation"))]
    let with_docs: Option<proc_macro2::TokenStream> = None;

    let where_clause_options = WhereClauseOptions::new(meta);
    let typed_impl = impl_typed(
        meta,
        &where_clause_options,
        quote! {
            let info = #obel_reflect_path::OpaqueInfo::new::<Self>() #with_docs;
            #obel_reflect_path::TypeInfo::Opaque(info)
        },
    );

    let type_path_impl = impl_type_path(meta);
    let full_reflect_impl = impl_full_reflect(meta, &where_clause_options);
    let common_methods = common_partial_reflect_methods(meta, || None, || None);
    let clone_fn = meta.attrs().get_clone_impl(obel_reflect_path);

    let apply_impl = if let Some(remote_ty) = meta.remote_ty() {
        let ty = remote_ty.type_path();
        quote! {
            if let #FQOption::Some(value) = <dyn #obel_reflect_path::PartialReflect>::try_downcast_ref::<#ty>(value) {
                *self = Self(#FQClone::clone(value));
                return #FQResult::Ok(());
            }
        }
    } else {
        quote! {
            if let #FQOption::Some(value) = <dyn #obel_reflect_path::PartialReflect>::try_downcast_ref::<Self>(value) {
                *self = #FQClone::clone(value);
                return #FQResult::Ok(());
            }
        }
    };

    #[cfg(not(feature = "functions"))]
    let function_impls = None::<proc_macro2::TokenStream>;
    #[cfg(feature = "functions")]
    let function_impls = crate::impls::impl_function_traits(meta, &where_clause_options);

    let (impl_generics, ty_generics, where_clause) = type_path.generics().split_for_impl();
    let where_reflect_clause = where_clause_options.extend_where_clause(where_clause);
    let get_type_registration_impl = meta.get_type_registration(&where_clause_options);

    quote! {
        #get_type_registration_impl

        #type_path_impl

        #typed_impl

        #full_reflect_impl

        #function_impls

        impl #impl_generics #obel_reflect_path::PartialReflect for #type_path #ty_generics #where_reflect_clause  {
            #[inline]
            fn get_represented_type_info(&self) -> #FQOption<&'static #obel_reflect_path::TypeInfo> {
                #FQOption::Some(<Self as #obel_reflect_path::Typed>::type_info())
            }

            #[inline]
            fn clone_value(&self) -> #obel_reflect_path::__macro_exports::alloc_utils::Box<dyn #obel_reflect_path::PartialReflect> {
                #obel_reflect_path::__macro_exports::alloc_utils::Box::new(#FQClone::clone(self))
            }

             #[inline]
            fn try_apply(
                &mut self,
                value: &dyn #obel_reflect_path::PartialReflect
            ) -> #FQResult<(), #obel_reflect_path::ApplyError> {
                #apply_impl

                #FQResult::Err(
                    #obel_reflect_path::ApplyError::MismatchedTypes {
                        from_type: ::core::convert::Into::into(#obel_reflect_path::DynamicTypePath::reflect_type_path(value)),
                        to_type: ::core::convert::Into::into(<Self as #obel_reflect_path::TypePath>::type_path()),
                    }
                )
            }

            #[inline]
            fn reflect_kind(&self) -> #obel_reflect_path::ReflectKind {
                #obel_reflect_path::ReflectKind::Opaque
            }

            #[inline]
            fn reflect_ref(&self) -> #obel_reflect_path::ReflectRef {
                #obel_reflect_path::ReflectRef::Opaque(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> #obel_reflect_path::ReflectMut {
                #obel_reflect_path::ReflectMut::Opaque(self)
            }

            #[inline]
            fn reflect_owned(self: #obel_reflect_path::__macro_exports::alloc_utils::Box<Self>) -> #obel_reflect_path::ReflectOwned {
                #obel_reflect_path::ReflectOwned::Opaque(self)
            }

            #common_methods

            #clone_fn
        }
    }
}
