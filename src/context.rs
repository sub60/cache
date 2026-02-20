use core::time::Duration;

use axum::http::{HeaderName, HeaderValue};

/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {
    /// The CDN that sits in front of the cache server.
    type Cdn: Cdn;

    /// Returns a reference to the CDN.
    fn cdn(&self) -> &Self::Cdn;
}

/// TODO: docs.
pub trait Cdn {
    /// TODO: docs.
    type NarInfoCacheHeaders: IntoIterator<Item = (HeaderName, HeaderValue)>;

    /// Returns the headers that should be included in a 200 OK response to the
    /// [`narinfo`](crate::routes::narinfo) route, given the specified cache
    /// duration.
    fn narinfo_cache_headers(
        &self,
        cache_duration: Duration,
    ) -> Self::NarInfoCacheHeaders;
}
