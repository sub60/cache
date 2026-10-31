//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

extern crate alloc;

mod cache_db;
mod cache_id;
mod cache_infos;
mod cache_name;
mod cdn;
mod clock;
mod context;
mod from_str_body;
mod namespace_cache_layer;
mod namespace_name;
pub mod nar_multiparts;
mod object_gateway;
mod object_storage;
mod route_layer;
mod router;
mod routes;
mod server_error_log_layer;

pub use cache_db::CacheDb;
pub use cache_id::{CacheId, CacheIdFromStrError};
pub use cache_infos::CacheInfos;
pub use cache_name::{CacheName, CacheNameFromStrError};
pub use cdn::Cdn;
pub use clock::Clock;
pub use context::Context;
pub use namespace_name::{NamespaceName, NamespaceNameFromStrError};
pub use object_gateway::{
    MultipartUploadUrlRequest,
    ObjectGateway,
    ObjectUrlRequest,
};
pub use object_storage::{
    CompleteMultipartUploadError,
    ObjectStorage,
    ObjectStorageKey,
};
pub use router::router;
