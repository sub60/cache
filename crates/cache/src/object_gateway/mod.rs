//! TODO: docs.

mod into_router;

use core::time::Duration;

use axum::http;
pub use into_router::IntoRouterObjectGateway;

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;

/// TODO: docs.
pub trait ObjectGateway: Sync {
    /// TODO: docs.
    type Storage: ObjectStorage;

    /// Returns a URL which is able to fulfill the given access request to an
    /// object in the storage.
    fn object_url(&self, access_request: StorageAccessRequest<'_>)
    -> http::Uri;

    /// TODO: docs.
    fn storage(&self) -> &Self::Storage;
}

/// TODO: docs.
pub struct StorageAccessRequest<'a> {
    /// The key of the object for which to generate a presigned URL.
    pub key: ObjectStorageKey<'a>,
    /// The HTTP method that the presigned URL should allow.
    pub method: http::Method,
    /// The TTL for the presigned URL.
    pub min_ttl: Duration,
}
