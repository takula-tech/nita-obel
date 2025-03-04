use crate::{derive_data::ReflectMeta, where_clause_options::WhereClauseOptions};
use quote::quote;

pub(crate) fn impl_get_ownership(
    meta: &ReflectMeta,
    where_clause_options: &WhereClauseOptions,
) -> proc_macro2::TokenStream {
    let obel_reflect = meta.obel_reflect_path();
    let type_path = meta.type_path();

    let (impl_generics, ty_generics, where_clause) = type_path.generics().split_for_impl();
    let where_reflect_clause = where_clause_options.extend_where_clause(where_clause);

    quote! {
        impl #impl_generics #obel_reflect::func::args::GetOwnership for #type_path #ty_generics #where_reflect_clause {
            fn ownership() -> #obel_reflect::func::args::Ownership {
                #obel_reflect::func::args::Ownership::Owned
            }
        }

        impl #impl_generics #obel_reflect::func::args::GetOwnership for &'_ #type_path #ty_generics #where_reflect_clause {
            fn ownership() -> #obel_reflect::func::args::Ownership {
                #obel_reflect::func::args::Ownership::Ref
            }
        }

        impl #impl_generics #obel_reflect::func::args::GetOwnership for &'_ mut #type_path #ty_generics #where_reflect_clause {
            fn ownership() -> #obel_reflect::func::args::Ownership {
                #obel_reflect::func::args::Ownership::Mut
            }
        }
    }
}
