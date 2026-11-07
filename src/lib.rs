//! Sub60's cache library.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

extern crate alloc;

mod cache_access_layer;
mod cache_control;
mod cache_db;
mod cache_id;
mod cache_infos;
mod ci_job;
mod clock;
mod commit_sha;
mod context;
mod endpoints;
mod from_str_body;
mod jwt;
mod nar_storage;
mod narinfo_storage;
mod oidc_verifier;
mod permission;
mod route_layer;
mod router;
mod serde_utils;
mod server_error_log_layer;
mod token;

pub use cache_control::CacheControl;
pub use cache_db::CacheDb;
pub use cache_id::{CacheId, CacheIdFromStrError};
pub use cache_infos::CacheInfos;
pub use ci_job::{CiJob, HostSoftware};
pub use clock::Clock;
pub use commit_sha::{CommitSha, CommitShaFromStrError};
pub use context::Context;
pub use nar_storage::{
    CompleteMultipartUploadError,
    CompletedMultipartUpload,
    NarStorage,
};
pub use narinfo_storage::NarInfoStorage;
pub use oidc_verifier::{OidcIssuer, OidcVerifier, Rs256Signature};
pub use permission::{Permission, Permissions};
pub use router::router;
pub use sub60_cache_api as api;
pub use token::{Token, TokenFormat, TokenHash, TokenKind};
