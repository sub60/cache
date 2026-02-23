use core::time::Duration;

use axum::http::{HeaderName, HeaderValue};

/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {
    /// The CDN that sits in front of the cache server.
    type Cdn: Cdn;

    /// Returns a reference to the CDN.
    fn cdn(&self) -> &Self::Cdn;

    /// TODO: docs.
    fn want_mass_query(&self) -> bool {
        true
    }
}

/// TODO: docs.
pub trait Cdn {
    /// TODO: docs.
    type CacheHeaders: IntoIterator<Item = (HeaderName, HeaderValue)>;

    /// Returns the headers to be included in a 200 OK response to make the CDN
    /// cache it for the specified duration.
    fn cache_headers(&self, cache_duration: Duration) -> Self::CacheHeaders;
}
