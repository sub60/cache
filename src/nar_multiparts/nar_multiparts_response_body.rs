use core::fmt;
use core::num::NonZeroU32;
use core::str::FromStr;

use crate::nar_multiparts::{FmtWrite, UploadId};

/// The type of error that can occur when parsing a string into a
/// [`NarMultipartsResponseBody`].
pub type NarMultipartsResponseBodyFromStrError = serde_json::Error;

/// The body of a `/nar-multiparts` response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NarMultipartsResponseBody {
    /// TODO: docs.
    pub upload_id: UploadId,

    /// TODO: docs.
    pub part_size: NonZeroU32,

    /// TODO: docs.
    pub parts_urls: [url::Url; 4],
}

impl fmt::Display for NarMultipartsResponseBody {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_writer(FmtWrite(f), &self).map_err(|_| fmt::Error)
    }
}

impl FromStr for NarMultipartsResponseBody {
    type Err = NarMultipartsResponseBodyFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
