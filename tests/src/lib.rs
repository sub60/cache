//! Integration tests for the Sub60 binary cache.

#![allow(incomplete_features)]
#![cfg_attr(not(test), allow(dead_code, unused_imports))]
#![feature(generic_const_exprs)]

mod auth;
mod cache_access;
mod cache_control;
mod common;
mod missing_narinfos;
mod multipart;
mod nar_get;
mod nar_put;
mod narinfo;
mod nix_copy;

pub use common::*;
