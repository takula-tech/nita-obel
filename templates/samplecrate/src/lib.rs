// - Includes the README content in the generated documentation (rustdoc)
// - Allows running doc tests from the README examples
// - Makes the README content available through cargo doc
// - Enables IDE documentation preview features
#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![doc(html_logo_url = "assets/icon.png", html_favicon_url = "assets/icon.png")]
#![no_std] // tells the compiler "don't automatically link std"
