use core::iter;
use core::time::Duration;

use axum::http::{HeaderName, HeaderValue, header};
use compact_str::format_compact;

use crate::context::Cdn;

/// The [`Cdn`] implementation for Cloudflare.
#[derive(Debug)]
pub struct CloudflareCdn;

impl Cdn for CloudflareCdn {
    type CacheHeaders = iter::Once<(HeaderName, HeaderValue)>;

    fn cache_headers(&self, cache_duration: Duration) -> Self::CacheHeaders {
        iter::once((
            header::CACHE_CONTROL,
            HeaderValue::from_str(&format_compact!(
                "public, max-age={}",
                cache_duration.as_secs()
            ))
            .expect("valid header value"),
        ))
    }
}
