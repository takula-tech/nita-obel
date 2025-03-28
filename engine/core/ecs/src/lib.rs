#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"

#[cfg(target_pointer_width = "16")]
compile_error!("obel_ecs cannot safely compile for a 16-bit platform.");

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// Required to make proc macros work in obel_ecs itself.
extern crate self as obel_ecs;

pub(crate) use checked_unwrap::*;
mod checked_unwrap;
pub mod error;
pub mod resource;
pub mod storage;
