//! Integration tests for the Sub60 binary cache.

#![allow(incomplete_features)]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]
#![feature(generic_const_exprs)]

mod common;
mod multipart;
mod narinfo;
mod nix_copy;

pub use common::*;
