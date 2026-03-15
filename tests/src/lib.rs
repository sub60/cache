//! Integration tests for the Sub60 binary cache.

#![allow(incomplete_features)]
#![allow(missing_docs)]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]
#![feature(generic_const_exprs)]

mod cache;
mod common;
#[cfg(feature = "server-prod")]
mod server_prod;
