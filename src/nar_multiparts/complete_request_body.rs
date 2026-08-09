use core::fmt;
use core::str::FromStr;

use nix_types::NarFileName;
use smallvec::SmallVec;

use crate::nar_multiparts::{Etag, FmtWrite};

/// The type of error that can occur when parsing a string into a
/// [`CompleteRequestBody`].
pub type CompleteRequestBodyFromStrError = serde_json::Error;

/// The body of a `/nar-multiparts/{uploadId}/complete` request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CompleteRequestBody {
    /// The list of [`Etag`]s returned by the `UploadPart` requests to the
    /// cache's object storage.
    ///
    /// The [`Etag`]s must be in order, i.e. the first one must be the one
    /// returned by uploading part 1, the second one by uploading part 2, etc.
    pub etags: SmallVec<[Etag; 8]>,

    /// The name of the NAR being uploaded.
    pub nar_file_name: NarFileName,
}

impl fmt::Display for CompleteRequestBody {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        serde_json::to_writer(FmtWrite(f), self).map_err(|_| fmt::Error)
    }
}

impl FromStr for CompleteRequestBody {
    type Err = CompleteRequestBodyFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}
