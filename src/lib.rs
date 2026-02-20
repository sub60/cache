//! Sub60's cache library.

#[cfg(feature = "cloudflare")]
pub mod cloudflare;
mod context;
mod router;
mod routes;
pub mod types;

pub use context::Context;
pub use router::router;
