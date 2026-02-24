//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

mod cache_ctx;
mod cdn;
#[cfg(feature = "cloudflare")]
pub mod cloudflare;
mod context;
mod layers;
mod object_storage;
mod router;
mod routes;
pub mod types;
mod user_ctx;

pub use cdn::Cdn;
pub use context::Context;
pub use object_storage::{ObjectStorage, PresignRequest};
pub use router::router;
