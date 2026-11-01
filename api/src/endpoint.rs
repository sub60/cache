use http::Method;

use crate::{CacheName, NamespaceName, PathParams};

/// An endpoint of the cache's HTTP API.
pub trait Endpoint {
    /// The method of the endpoint's requests.
    const METHOD: Method;

    /// The template of the endpoint's path.
    const PATH: &'static str = <Self::Path as PathParams>::PATH;

    /// The parameters in the endpoint's path.
    type Path: PathParams;

    /// The body of the endpoint's requests.
    type Request;

    /// The endpoint's successful responses.
    type Response;
}

/// The path parameters identifying a cache, which the paths of all the cache's
/// endpoints start with.
#[derive(Debug, Clone, PartialEq, Eq, PathParams)]
#[route("/{namespace}/{cache}")]
pub struct CachePath {
    /// The namespace owning the cache.
    pub namespace: NamespaceName,

    /// The cache's name.
    pub cache: CacheName,
}

/// The prefix of the paths of the endpoints that aren't part of a cache.
///
/// It can't be a namespace name, so it never clashes with a cache's paths.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PathParams)]
#[route("/-/api/v1")]
pub struct ApiPath;
