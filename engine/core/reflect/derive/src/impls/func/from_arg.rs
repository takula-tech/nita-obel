use crate::{derive_data::ReflectMeta, where_clause_options::WhereClauseOptions};
use obel_reflect_utils::FQResult;
use quote::quote;

pub(crate) fn impl_from_arg(
    meta: &ReflectMeta,
    where_clause_options: &WhereClauseOptions,
) -> proc_macro2::TokenStream {
    let obel_reflect = meta.obel_reflect_path();
    let type_path = meta.type_path();

    let (impl_generics, ty_generics, where_clause) = type_path.generics().split_for_impl();
    let where_reflect_clause = where_clause_options.extend_where_clause(where_clause);

    quote! {
        impl #impl_generics #obel_reflect::func::args::FromArg for #type_path #ty_generics #where_reflect_clause {
            type This<'from_arg> = #type_path #ty_generics;
            fn from_arg(arg: #obel_reflect::func::args::Arg) -> #FQResult<Self::This<'_>, #obel_reflect::func::args::ArgError> {
                arg.take_owned()
            }
        }

        impl #impl_generics #obel_reflect::func::args::FromArg for &'static #type_path #ty_generics #where_reflect_clause {
            type This<'from_arg> = &'from_arg #type_path #ty_generics;
            fn from_arg(arg: #obel_reflect::func::args::Arg) -> #FQResult<Self::This<'_>, #obel_reflect::func::args::ArgError> {
                arg.take_ref()
            }
        }

        impl #impl_generics #obel_reflect::func::args::FromArg for &'static mut #type_path #ty_generics #where_reflect_clause {
            type This<'from_arg> = &'from_arg mut #type_path #ty_generics;
            fn from_arg(arg: #obel_reflect::func::args::Arg) -> #FQResult<Self::This<'_>, #obel_reflect::func::args::ArgError> {
                arg.take_mut()
            }
        }
    }
}
