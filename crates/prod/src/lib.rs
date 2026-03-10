//! Sub60's production cache implementation backed by Cloudflare.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod cdn;
mod r2;

pub use cdn::CloudflareCdn;
pub use r2::R2ObjectStorage;
