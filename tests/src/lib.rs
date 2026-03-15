//! Integration tests for the Sub60 binary cache.

#![allow(incomplete_features)]
#![allow(missing_docs)]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]
#![feature(generic_const_exprs)]

mod common;

mod cache;

#[cfg(all(nix_in_path, workerd_in_path))]
mod server_prod;
