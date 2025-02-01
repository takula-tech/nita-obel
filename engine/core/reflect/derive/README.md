# Obel Reflect Derive

[![license](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/obelengine/obel#license)
[![crates.io](https://img.shields.io/crates/v/obel.svg)](https://crates.io/crates/obel)
[![downloads](https://img.shields.io/crates/d/obel.svg)](https://crates.io/crates/obel)
[![docs.rs](https://docs.rs/obel/badge.svg)](https://docs.rs/obel/latest/obel/)
[![discord.online](https://img.shields.io/discord/1335036405788971020.svg?label=&logo=discord&logoColor=ffffff&color=7389D8)](https://discord.gg/3jq8js8u)

This crate contains macros used by Bevy's `Reflect` API.

The main export of this crate is the derive macro for [`Reflect`]. This allows
types to easily implement `Reflect` along with other `bevy_reflect` traits,
such as `Struct`, `GetTypeRegistration`, and more— all with a single derive!

Some other noteworthy exports include the derive macros for [`FromReflect`] and
[`TypePath`], as well as the [`reflect_trait`] attribute macro.

[`Reflect`]: crate::derive_reflect
[`FromReflect`]: crate::derive_from_reflect
[`TypePath`]: crate::derive_type_path
[`reflect_trait`]: macro@reflect_trait
