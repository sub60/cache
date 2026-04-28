use core::iter;
use core::time::Duration;

use axum::http::{HeaderName, HeaderValue};
use cauchy::Debug;
use sub60_cache::Cdn;

/// A [`Cdn`] implementation that does nothing.
#[derive(Debug, Copy, Clone, Default)]
pub struct NoOpCdn {}

impl Cdn for NoOpCdn {
    type CacheHeaders = iter::Empty<(HeaderName, HeaderValue)>;

    fn cache_headers(&self, _: Duration) -> Self::CacheHeaders {
        iter::empty()
    }

    fn cache_headers_no_cache(&self) -> Self::CacheHeaders {
        iter::empty()
    }
}
