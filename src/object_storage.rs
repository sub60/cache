use core::error::Error;
use core::fmt;
use core::time::Duration;

use axum::http;

use crate::types::{NarFileName, NixStoreHash};

/// TODO: docs.
pub trait ObjectStorage: Sync {
    /// TODO: docs.
    type Key: From<NarFileName> + From<NixStoreHash> + Send;

    /// TODO: docs.
    type Error: Error + Send;

    /// Returns whether the object storage contains the given key.
    fn contains_key(
        &self,
        key: &Self::Key,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Returns whether the object storage contains the given key.
    fn get(
        &self,
        key: &Self::Key,
    ) -> impl Future<Output = Result<Option<impl AsRef<[u8]>>, Self::Error>> + Send;

    /// Generates a presigned URL for the given [`PresignRequest`], returning an
    /// opaque type containing the path and query parameters for the presigned
    /// URL.
    fn presign(
        &self,
        presign_request: PresignRequest<Self::Key>,
    ) -> impl fmt::Debug + Into<http::HeaderValue>;
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
