#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod bundle;
mod cmpt;
mod event;
mod from_world;
mod label;
mod param;
mod query;
mod resource;

use crate::bundle::derive_bundle_impl;
use crate::event::derive_event_impl;
use crate::from_world::derive_from_world_impl;
use crate::label::{derive_schedule_label_impl, derive_system_set_label_impl};
use crate::param::derive_system_param_impl;
use crate::query::{
    derive_query_data_impl, derive_query_filter_impl, derive_states_impl, derive_substates_impl,
    derive_visit_entities_impl, derive_visit_entities_mut_impl,
};
use crate::resource::derive_resource_impl;
use cmpt::derive_component_impl;
use obel_reflect_utils::ObelManifest;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub(crate) fn obel_ecs_path() -> syn::Path {
    ObelManifest::shared().get_path("obel_ecs")
}

/// Implement `Bundle` to make it easy to create a bundle of components
#[proc_macro_derive(Bundle, attributes(bundle))]
pub fn derive_bundle(input: TokenStream) -> TokenStream {
    derive_bundle_impl(TokenStream2::from(input)).into()
}

/// Implement `VisitEntitiesMut` to loop through mut entities
#[proc_macro_derive(VisitEntitiesMut, attributes(visit_entities))]
pub fn derive_visit_entities_mut(input: TokenStream) -> TokenStream {
    derive_visit_entities_mut_impl(TokenStream2::from(input)).into()
}

/// Implement `VisitEntitiesMut` to loop through shared entities
#[proc_macro_derive(VisitEntities, attributes(visit_entities))]
pub fn derive_visit_entities(input: TokenStream) -> TokenStream {
    derive_visit_entities_impl(TokenStream2::from(input)).into()
}

/// Implement `SystemParam` to use a struct as a parameter in a system
#[proc_macro_derive(SystemParam, attributes(system_param))]
pub fn derive_system_param(input: TokenStream) -> TokenStream {
    derive_system_param_impl(TokenStream2::from(input)).into()
}

/// Implement `QueryData` to use a struct as a data parameter in a query
#[proc_macro_derive(QueryData, attributes(query_data))]
pub fn derive_query_data(input: TokenStream) -> TokenStream {
    derive_query_data_impl(TokenStream2::from(input)).into()
}

/// Implement `QueryFilter` to use a struct as a filter parameter in a query
#[proc_macro_derive(QueryFilter, attributes(query_filter))]
pub fn derive_query_filter(input: TokenStream) -> TokenStream {
    derive_query_filter_impl(TokenStream2::from(input)).into()
}

/// Derive macro generating an impl of the trait `ScheduleLabel`.
///
/// This does not work for unions.
#[proc_macro_derive(ScheduleLabel)]
pub fn derive_schedule_label(input: TokenStream) -> TokenStream {
    derive_schedule_label_impl(TokenStream2::from(input)).into()
}

/// Derive macro generating an impl of the trait `SystemSet`.
///
/// This does not work for unions.
#[proc_macro_derive(SystemSet)]
pub fn derive_system_set_label(input: TokenStream) -> TokenStream {
    derive_system_set_label_impl(TokenStream2::from(input)).into()
}

/// Implement `Event` to use a struct as event
#[proc_macro_derive(Event, attributes(event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    derive_event_impl(TokenStream2::from(input)).into()
}

/// Implement `Resource` to use a struct as resource
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    derive_resource_impl(TokenStream2::from(input)).into()
}

/// Implement `Component` to use a struct as component
#[proc_macro_derive(
    Component,
    attributes(component, require, relationship, relationship_target, entities)
)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    derive_component_impl(TokenStream2::from(input)).into()
}

/// Implement `States` to use a struct as states
#[proc_macro_derive(States)]
pub fn derive_states(input: TokenStream) -> TokenStream {
    derive_states_impl(TokenStream2::from(input)).into()
}

/// Implement `SubStates` to use a struct as subStates
#[proc_macro_derive(SubStates, attributes(source))]
pub fn derive_substates(input: TokenStream) -> TokenStream {
    derive_substates_impl(TokenStream2::from(input)).into()
}

/// Implement `FromWorld` to use a struct
#[proc_macro_derive(FromWorld, attributes(from_world))]
pub fn derive_from_world(input: TokenStream) -> TokenStream {
    derive_from_world_impl(TokenStream2::from(input)).into()
}
