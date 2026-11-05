//! The `HEAD /{namespace}/{cache}/{narinfo_file_name}` endpoint.

use http::{Method, StatusCode};

use crate::endpoints::narinfo_get;

/// Checks whether a NARInfo exists.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
pub type Path = narinfo_get::Path;

/// The response to the endpoint's requests.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Response {
    /// The NARInfo exists.
    Exists,

    /// The NARInfo doesn't exist.
    NotFound,
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::HEAD;

    type Path = Path;
    type Request = ();
    type Response = Response;
}

impl Response {
    /// Returns the response's status code.
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Exists => StatusCode::OK,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
