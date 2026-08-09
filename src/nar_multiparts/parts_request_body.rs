use core::num::NonZeroU16;
use core::{fmt, str};

use nix_types::NarFileName;
use smallvec::SmallVec;

use crate::nar_multiparts::FmtWrite;

/// The type of error that can occur when parsing a string into a
/// [`PartsRequestBody`].
pub type PartsRequestBodyFromStrError = serde_json::Error;

/// The body of a `/nar-multiparts/{uploadId}/parts` request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PartsRequestBody {
    /// The list of part numbers to request upload URLs for. The numbers can be
    /// in any order, but there shouldn't be any duplicates.
    pub part_numbers: SmallVec<[NonZeroU16; 8]>,

    /// The name of the NAR being uploaded.
    pub nar_file_name: NarFileName,
}

impl fmt::Display for PartsRequestBody {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_writer(FmtWrite(f), self).map_err(|_| fmt::Error)
    }
}

impl str::FromStr for PartsRequestBody {
    type Err = PartsRequestBodyFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
