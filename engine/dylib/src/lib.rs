#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc(
    html_logo_url = "https://obelengine.org/assets/icon.png",
    html_favicon_url = "https://obelengine.org/assets/icon.png"
)]
#![no_std] // tells the compiler "don't automatically link std"

// Force linking of the main obel crate
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use obel_api;
