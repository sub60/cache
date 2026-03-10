//! Sub60's production cache implementation backed by Cloudflare.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod cdn;
mod object_storage;

pub use cdn::CloudflareCdn;
pub use object_storage::R2ObjectStorage;
