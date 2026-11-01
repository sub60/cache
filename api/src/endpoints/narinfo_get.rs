//! The `GET /{namespace}/{cache}/{narinfo_file_name}` endpoint.

use http::{Method, StatusCode};
use nix_types::{NarInfo, NarInfoFileName};

use crate::{CachePath, PathParams};

/// Gets a NARInfo.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::AsRef, PathParams)]
#[route(CachePath + "/{narinfo_file_name}")]
pub struct Path {
    /// The cache the NARInfo is in.
    #[as_ref]
    pub cache: CachePath,

    /// The NARInfo's file name.
    pub narinfo_file_name: NarInfoFileName,
}

/// The response to the endpoint's requests.
#[derive(Debug, PartialEq, Eq)]
#[expect(clippy::large_enum_variant)]
pub enum Response {
    /// The NARInfo, signed with the cache's key.
    Found(NarInfo<String>),

    /// The NARInfo doesn't exist.
    NotFound,
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::GET;

    type Path = Path;
    type Request = ();
    type Response = Response;
}

impl Response {
    /// Returns the response's status code.
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Found(_) => StatusCode::OK,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
