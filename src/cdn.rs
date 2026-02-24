use core::time::Duration;

use axum::http::{HeaderName, HeaderValue};

/// TODO: docs.
pub trait Cdn {
    /// TODO: docs.
    type CacheHeaders: IntoIterator<Item = (HeaderName, HeaderValue)>;

    /// Returns the headers to be included in a 200 OK response to make the CDN
    /// cache it for the specified duration.
    fn cache_headers(&self, cache_duration: Duration) -> Self::CacheHeaders;
}
