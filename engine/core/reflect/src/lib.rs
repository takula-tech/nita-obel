//! obel_reflect

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
// The following three declarations (#![no_std], extern crate alloc, extern crate std)
// are required for proper integration with the no_std obel_platform crate.
// Without these declarations, while local compilation and tests succeed,
// the release build fails with import errors for obel_platform's boxed and vec types
// with the below error:
// Compiling obel_reflect_derive v0.0.5 (/home/runner/work/nita-obel/nita-obel/target/package/obel_reflect_derive-0.0.5)
// error[E0432]: unresolved import `obel_platform::boxed`
//  --> src/remote.rs:9:21
//   |
// 9 | use obel_platform::{boxed::Box, vec::Vec};
//   |                     ^^^^^ could not find `boxed` in `obel_platform`
#![no_std] // tells the compiler "don't automatically link std"
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

extern crate proc_macro;
