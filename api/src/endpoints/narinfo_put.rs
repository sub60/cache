//! The `PUT /{namespace}/{cache}/{narinfo_file_name}` endpoint.

use http::{Method, StatusCode};
use nix_types::NarInfo;

use crate::endpoints::narinfo_get;

/// Uploads a NARInfo.
///
/// Its NAR must already have been uploaded.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
pub type Path = narinfo_get::Path;

/// The response to the endpoint's requests.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Response {
    /// The NARInfo was stored.
    Created,

    /// The NARInfo already existed, and was left as is.
    AlreadyExists,
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::PUT;

    type Path = Path;
    type Request = NarInfo<String>;
    type Response = Response;
}

impl Response {
    /// Returns the response's status code.
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Created => StatusCode::CREATED,
            Self::AlreadyExists => StatusCode::OK,
        }
    }
}
