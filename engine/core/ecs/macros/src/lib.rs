#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod cmpt;

use obel_reflect_utils::ObelManifest;

pub(crate) fn obel_ecs_path() -> syn::Path {
    ObelManifest::shared().get_path("obel_ecs")
}
