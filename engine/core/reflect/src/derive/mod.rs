//! This module contains macros used by Bevy's `Reflect` API.
//!
//! The main export of this crate is the derive macro for [`Reflect`]. This allows
//! types to easily implement `Reflect` along with other `bevy_reflect` traits,
//! such as `Struct`, `GetTypeRegistration`, and more— all with a single derive!
//!
//! Some other noteworthy exports include the derive macros for [`FromReflect`] and
//! [`TypePath`], as well as the [`reflect_trait`] attribute macro.
//!
//! [`Reflect`]: crate::derive_reflect
//! [`FromReflect`]: crate::derive_from_reflect
//! [`TypePath`]: crate::derive_type_path
//! [`reflect_trait`]: macro@reflect_trait
