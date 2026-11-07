//! The `POST /{namespace}/{cache}/nar-multiparts/{upload_id}/parts` endpoint.

use core::fmt;
use core::num::NonZeroU16;
use core::str::FromStr;
use core::time::Duration;

use http::Method;
use smallvec::SmallVec;
use url::Url;

use crate::endpoints::nar_multiparts::UploadId;
use crate::fmt_write::FmtWrite;
use crate::{CachePath, PathParams};

/// Gets the URLs to upload some parts of a multipart upload to.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::AsRef, PathParams)]
#[route(CachePath + "/nar-multiparts/{upload_id}/parts")]
pub struct Path<Id = UploadId> {
    /// The cache the NAR is uploaded to.
    #[as_ref]
    pub cache: CachePath,

    /// The ID of the upload.
    pub upload_id: Id,
}

/// The body of a `/nar-multiparts/{uploadId}/parts` request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequestBody {
    /// The list of part numbers to request upload URLs for. The numbers can be
    /// in any order, but there shouldn't be any duplicates.
    pub part_numbers: SmallVec<[NonZeroU16; 8]>,
}

/// The type of error that can occur when parsing a string into a
/// [`RequestBody`].
pub type RequestBodyFromStrError = serde_json::Error;

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

/// The type of error that can occur when parsing a string into a
/// [`ResponseBody`].
pub type ResponseBodyFromStrError = serde_json::Error;

/// The body of a `/nar-multiparts/{uploadId}/parts` response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseBody {
    /// The pre-signed URLs whose order matches the one of the
    /// [part numbers](RequestBody::part_numbers) in the
    /// [`RequestBody`], i.e. the first URL is for the first part number,
    /// the second URL is for the part number item, and so on.
    pub urls: SmallVec<[Url; 8]>,

    /// The expiration timestamp of this batch of upload URLs, expressed as the
    /// elapsed time since the UNIX epoch.
    #[serde(with = "crate::serde_utils::secs")]
    pub expiration_ts: Duration,
}

impl fmt::Display for ResponseBody {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_writer(FmtWrite(f), self).map_err(|_| fmt::Error)
    }
}

impl FromStr for ResponseBody {
    type Err = ResponseBodyFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::POST;

    type Path = Path;
    type Request = RequestBody;
    type Response = ResponseBody;
}
