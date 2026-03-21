//! TODO: docs.

mod into_router;

use core::time::Duration;

use axum::http;
pub use into_router::IntoRouterObjectGateway;
use types::NarFileName;

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;

/// TODO: docs.
pub trait ObjectGateway {
    /// TODO: docs.
    type Storage: ObjectStorage;

    /// Returns the URL at which the NAR corresponding to the given file name
    /// can be accessed, if it exists.
    fn nar_url<const IS_FOR_PUT: bool>(
        &self,
        nar_filename: &NarFileName,
    ) -> http::Uri;

    /// TODO: docs.
    fn storage(&self) -> &Self::Storage;

    /// Generates a presigned URL for the given [`PresignRequest`].
    ///
    /// The returned URL contains embedded authentication parameters (e.g.
    /// AWS SigV4 query parameters) that grant temporary access to the
    /// specified object for the given HTTP method.
    fn presign(
        &self,
        _presign_request: PresignRequest<ObjectStorageKey<'_>>,
    ) -> http::Uri {
        unimplemented!(
            "presigning requires S3-compatible credentials; \
             not available for this ObjectGateway implementation"
        )
    }
}

/// TODO: docs.
pub struct PresignRequest<K> {
    /// The key of the object for which to generate a presigned URL.
    pub key: K,
    /// The HTTP method that the presigned URL should allow.
    pub method: http::Method,
    /// The content type that the presigned URL should allow, if any.
    pub content_type: Option<http::HeaderValue>,
    /// The TTL for the presigned URL, if any.
    pub ttl: Option<Duration>,
}
