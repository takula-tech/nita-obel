use crate::obel_ecs_path;
use obel_reflect_utils::derive_label;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{DeriveInput, parse2};

pub fn derive_system_set_label_impl(input: TokenStream) -> TokenStream {
    let input = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };
    let mut trait_path = obel_ecs_path();
    trait_path.segments.push(format_ident!("schedule").into());
    let mut dyn_eq_path = trait_path.clone();
    trait_path.segments.push(format_ident!("SystemSet").into());
    dyn_eq_path.segments.push(format_ident!("DynEq").into());
    derive_label(input, "SystemSet", &trait_path, &dyn_eq_path)
}

pub fn derive_schedule_label_impl(input: TokenStream) -> TokenStream {
    let input = match parse2::<DeriveInput>(input) {
        Ok(ast) => ast,
        Err(e) => return e.into_compile_error(),
    };
    let mut trait_path = obel_ecs_path();
    trait_path.segments.push(format_ident!("schedule").into());
    let mut dyn_eq_path = trait_path.clone();
    trait_path.segments.push(format_ident!("ScheduleLabel").into());
    dyn_eq_path.segments.push(format_ident!("DynEq").into());
    derive_label(input, "ScheduleLabel", &trait_path, &dyn_eq_path)
}
