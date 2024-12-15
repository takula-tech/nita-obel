// FIXME(15321): solve CI failures, then replace with `#![expect()]`.
#![allow(missing_docs, reason = "Not all docs are written yet, see #3492.")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://obelengine.org/assets/icon.png",
    html_favicon_url = "https://obelengine.org/assets/icon.png"
)]

//! This crate provides a straightforward solution for integrating diagnostics in the [obel game engine](https://obelengine.org/).
//! It allows users to easily add diagnostic functionality to their obel applications, enhancing
//! their ability to monitor and optimize their game's.

extern crate alloc;
