//! The `POST /{namespace}/{cache}/nar-multiparts/{upload_id}/complete` endpoint.

use core::fmt;
use core::str::FromStr;

use http::{Method, StatusCode};
use nix_types::NarFileName;
use smallvec::SmallVec;

use crate::endpoints::nar_multiparts::UploadId;
use crate::fmt_write::FmtWrite;
use crate::{CachePath, Etag, PathParams};

/// Completes a multipart upload.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
#[derive(Debug, Clone, PartialEq, Eq, PathParams)]
#[route(CachePath + "/nar-multiparts/{upload_id}/complete")]
pub struct Path<Id = UploadId> {
    /// The cache the NAR is uploaded to.
    pub cache: CachePath,

    /// The ID of the upload.
    pub upload_id: Id,
}

/// The response to the endpoint's requests.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Response {
    /// The NAR was stored.
    Created,

    /// The NAR already existed, e.g. because another upload of it completed
    /// first, and this upload was discarded.
    AlreadyExists,
}

/// The type of error that can occur when parsing a string into a
/// [`RequestBody`].
pub type RequestBodyFromStrError = serde_json::Error;

/// The body of a `/nar-multiparts/{uploadId}/complete` request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequestBody {
    /// The list of [`Etag`]s returned by the `UploadPart` requests to the
    /// cache's object storage.
    ///
    /// The [`Etag`]s must be in order, i.e. the first one must be the one
    /// returned by uploading part 1, the second one by uploading part 2, etc.
    pub etags: SmallVec<[Etag; 8]>,

    /// The name of the NAR being uploaded.
    pub nar_file_name: NarFileName,
}

impl fmt::Display for RequestBody {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_writer(FmtWrite(f), self).map_err(|_| fmt::Error)
    }
}

impl FromStr for RequestBody {
    type Err = RequestBodyFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::POST;

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
            Self::AlreadyExists => StatusCode::OK,
        }
    }
}
