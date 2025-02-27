#![allow(dead_code)]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate proc_macro;

mod attr;
mod ident;
mod meta;
mod result_sifter;
mod string_expr;
