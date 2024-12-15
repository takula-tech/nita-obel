#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![expect(
    unsafe_code,
    reason = "Some utilities, such as futures and cells, require unsafe code."
)]
#![doc(
    html_logo_url = "https://obelengine.org/assets/icon.png",
    html_favicon_url = "https://obelengine.org/assets/icon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]

//! General utilities for first-party [obel] engine crates.
//!
//! [obel]: https://obelengine.org/

#[cfg(feature = "alloc")]
extern crate alloc;
