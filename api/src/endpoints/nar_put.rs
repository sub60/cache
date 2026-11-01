//! The `PUT /{namespace}/{cache}/nar/{nar_file_name}` endpoint.

use http::{Method, StatusCode, Uri};

use crate::endpoints::nar_head;

/// Uploads a NAR.
///
/// The server doesn't store the NAR itself, but redirects the request to where
/// it should be uploaded.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// The NAR already exists, so it doesn't need to be uploaded.
    AlreadyExists,

    /// The NAR should be uploaded to the given URL, which can be relative to
    /// the request's.
    Redirect(Uri),
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
            Self::AlreadyExists => StatusCode::NO_CONTENT,
            Self::Redirect(_) => StatusCode::TEMPORARY_REDIRECT,
        }
    }
}
