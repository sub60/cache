//! The types clients of Sub60's cache use to talk to it.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

extern crate alloc;
extern crate self as sub60_cache_api;

mod cache_name;
mod endpoint;
pub mod endpoints;
mod etag;
mod fmt_write;
mod namespace_name;
mod path_params;
mod serde_utils;

pub use cache_name::{CacheName, CacheNameFromStrError};
pub use endpoint::{ApiPath, CachePath, Endpoint};
pub use etag::{Etag, EtagFromStrError};
pub use namespace_name::{NamespaceName, NamespaceNameFromStrError};
#[doc(hidden)]
pub use path_params::__private;
#[cfg(feature = "axum")]
pub use path_params::PathParamsRejection;
pub use path_params::{PathParams, PathParamsError};
