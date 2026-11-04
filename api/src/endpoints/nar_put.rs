//! The `PUT /{namespace}/{cache}/nar/{nar_file_name}` endpoint.

use http::{Method, StatusCode};

use crate::endpoints::nar_head;

/// Uploads a NAR.
///
/// The server refuses NARs whose SHA-256 hash isn't the one in their file name
/// with a `400 Bad Request`.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
pub type Path = nar_head::Path;

/// The body of the endpoint's requests, i.e. the NAR's bytes.
///
/// It's only a marker, since clients stream the NAR instead of building it
/// in memory.
#[derive(Debug, Copy, Clone)]
pub struct RequestBody;

/// The response to the endpoint's requests.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Response {
    /// The NAR was stored.
    Created,

    /// The NAR already existed, and was left as is.
    AlreadyExists,
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::PUT;

    type Path = Path;
    type Request = RequestBody;
    type Response = Response;
}

impl Response {
    /// Returns the response's status code.
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Created => StatusCode::CREATED,
            Self::AlreadyExists => StatusCode::NO_CONTENT,
        }
    }
}
