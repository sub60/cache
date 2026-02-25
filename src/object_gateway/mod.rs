//! TODO: docs.

mod loopback;
mod r2;

use core::fmt;
use core::time::Duration;

use axum::http;
pub use loopback::LoopbackObjectGateway;

use crate::ObjectStorage;

/// TODO: docs.
pub trait ObjectGateway {
    /// TODO: docs.
    type Storage: ObjectStorage;

    /// Returns the URL at which the object corresponding to the given key can
    /// be accessed, if it exists.
    fn object_url(
        &self,
        key: <Self::Storage as ObjectStorage>::Key<'_>,
    ) -> http::Uri;

    /// TODO: docs.
    fn storage(&self) -> &Self::Storage;

    /// Generates a presigned URL for the given [`PresignRequest`], returning an
    /// opaque type containing the path and query parameters for the presigned
    /// URL.
    fn presign(
        &self,
        _presign_request: PresignRequest<
            <Self::Storage as ObjectStorage>::Key<'_>,
        >,
    ) -> impl fmt::Debug + Into<http::HeaderValue> {
        fn cast<T>(panic_msg: impl fmt::Display) -> T {
            unimplemented!("{panic_msg}")
        }
        cast::<http::HeaderValue>(
            "probably not usable since it buckets are either public or private
             (without granularity), and private buckets mean we can't use a \
             CDN, while for public buckets presigned URLs are a security \
             theater",
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
