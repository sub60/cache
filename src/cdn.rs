//! TODO: docs.

use core::time::Duration;

use axum::http::{HeaderName, HeaderValue};

/// TODO: docs.
pub trait Cdn {
    /// TODO: docs.
    type CacheHeaders: IntoIterator<Item = (HeaderName, HeaderValue)>;

    /// Returns the headers to be included in an HTTP response to make the CDN
    /// cache it for the specified duration.
    fn cache_headers(&self, cache_duration: Duration) -> Self::CacheHeaders;

    /// Returns the headers to be included in an HTTP response to make the CDN
    /// not cache it at all.
    fn cache_headers_no_cache(&self) -> Self::CacheHeaders;
}
