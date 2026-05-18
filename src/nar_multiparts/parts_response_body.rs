use core::{fmt, str};

use smallvec::SmallVec;
use url::Url;

use crate::nar_multiparts::FmtWrite;
#[expect(unused, reason = "used in doc comments")]
use crate::nar_multiparts::PartsRequestBody;

/// The type of error that can occur when parsing a string into a
/// [`PartsResponseBody`].
pub type PartsResponseBodyFromStrError = serde_json::Error;

/// The body of a `/nar-multiparts/{uploadId}/parts` response.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PartsResponseBody {
    /// The pre-signed URLs whose order matches the one of the
    /// [part numbers](PartsRequestBody::part_numbers) in the
    /// [`PartsRequestBody`], i.e. the first URL is for the first part number
    /// the second URL is for the part number item, and so on.
    pub urls: SmallVec<[Url; 8]>,
}

impl fmt::Display for PartsResponseBody {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_writer(FmtWrite(f), self).map_err(|_| fmt::Error)
    }
}

impl str::FromStr for PartsResponseBody {
    type Err = PartsResponseBodyFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
