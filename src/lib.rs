//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(duration_constructors)]

extern crate alloc;

mod cache_access_layer;
mod cache_db;
mod cache_id;
mod cache_infos;
mod cdn;
mod ci_job;
mod clock;
mod commit_sha;
mod context;
mod endpoints;
mod from_str_body;
mod jwt;
mod object_gateway;
mod object_storage;
mod oidc_verifier;
mod permission;
mod route_layer;
mod router;
mod serde_utils;
mod server_error_log_layer;
mod token;

pub use cache_db::CacheDb;
pub use cache_id::{CacheId, CacheIdFromStrError};
pub use cache_infos::CacheInfos;
pub use cdn::Cdn;
pub use ci_job::{CiJob, HostSoftware};
pub use clock::Clock;
pub use commit_sha::{CommitSha, CommitShaFromStrError};
pub use context::Context;
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
pub use permission::Permission;
pub use router::router;
pub use sub60_cache_api as api;
pub use token::{Token, TokenFormat, TokenHash, TokenKind};
