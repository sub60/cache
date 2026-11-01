//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

extern crate alloc;

pub mod auth;
mod cache_db;
mod cache_id;
mod cache_infos;
mod cache_name;
mod cdn;
mod ci_job;
mod clock;
mod commit_sha;
mod context;
mod from_str_body;
mod jwt;
mod namespace_cache_layer;
mod namespace_name;
pub mod nar_multiparts;
mod object_gateway;
mod object_storage;
mod oidc_verifier;
mod route_layer;
mod router;
mod routes;
mod serde_utils;
mod server_error_log_layer;
mod token;

pub use cache_db::CacheDb;
pub use cache_id::{CacheId, CacheIdFromStrError};
pub use cache_infos::CacheInfos;
pub use cache_name::{CacheName, CacheNameFromStrError};
pub use cdn::Cdn;
pub use ci_job::{CiJob, HostSoftware};
pub use clock::Clock;
pub use commit_sha::{CommitSha, CommitShaFromStrError};
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
pub use oidc_verifier::{OidcIssuer, OidcVerifier, Rs256Signature};
pub use router::router;
pub use token::{Token, TokenFormat, TokenHash, TokenKind};
