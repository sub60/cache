//! The `GET /{namespace}/{cache}/nix-cache-info` endpoint.

use http::Method;
use nix_types::NixCacheInfo;
use smol_str::SmolStr;

use crate::{CachePath, PathParams};

/// Gets a cache's `nix-cache-info`.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::AsRef, PathParams)]
#[route(CachePath + "/nix-cache-info")]
pub struct Path {
    /// The cache.
    #[as_ref]
    pub cache: CachePath,
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::GET;

    type Path = Path;
    type Request = ();
    type Response = NixCacheInfo<SmolStr>;
}
