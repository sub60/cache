//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

#[cfg(feature = "cloudflare")]
pub mod cloudflare;
mod context;
mod router;
mod routes;
pub mod types;

pub use context::Context;
pub use router::router;
