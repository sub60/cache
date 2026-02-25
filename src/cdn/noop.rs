use core::iter;
use core::time::Duration;

use axum::http::{HeaderName, HeaderValue};

use crate::Cdn;

/// A [`Cdn`] implementation that does nothing.
#[derive(Debug)]
pub struct NoOpCdn;

impl Cdn for NoOpCdn {
    type CacheHeaders = iter::Empty<(HeaderName, HeaderValue)>;

    fn cache_headers(&self, _: Duration) -> Self::CacheHeaders {
        iter::empty()
    }

    fn cache_headers_no_cache(&self) -> Self::CacheHeaders {
        iter::empty()
    }
}
