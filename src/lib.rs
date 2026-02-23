//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

mod cache_ctx;
#[cfg(feature = "cloudflare")]
pub mod cloudflare;
mod context;
mod layers;
mod router;
mod routes;
pub mod types;
mod user_ctx;

pub use context::Context;
pub use router::router;
