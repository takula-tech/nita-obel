#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod borrow;
pub mod boxed;
pub mod collections;
pub mod hash;
pub mod string;
pub mod sync;
pub mod time;
pub mod utils;
pub mod vec;
