//! TODO: docs.

use axum::http::HeaderValue;

pub(crate) mod nar_get;
pub(crate) mod nar_put;
pub(crate) mod narinfo_get;
pub(crate) mod narinfo_put;
pub(crate) mod nix_cache_info;

const NAR_CONTENT_TYPE: HeaderValue =
    HeaderValue::from_static("application/x-nix-nar");
