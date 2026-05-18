//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

extern crate alloc;

mod cache_db;
mod cache_infos;
mod cache_name;
mod cdn;
mod context;
mod from_str_body;
pub mod nar_multiparts;
mod object_gateway;
mod object_storage;
mod route_layer;
mod router;
mod routes;
mod user_cache_layer;
mod user_name;

pub use cache_db::CacheDb;
pub use cache_infos::CacheInfos;
pub use cache_name::{CacheName, CacheNameFromStrError};
pub use cdn::Cdn;
pub use context::Context;
pub use object_gateway::{
    MultipartUploadUrlRequest,
    ObjectGateway,
    ObjectUrlRequest,
};
pub use object_storage::{ObjectStorage, ObjectStorageKey};
pub use router::router;
pub use user_name::{UserName, UserNameFromStrError};
