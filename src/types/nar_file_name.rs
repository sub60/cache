use core::str::FromStr;
use std::borrow::Cow;

use crate::types::{
    CompressionExtension,
    CompressionExtensionFromStrError,
    Nix32HashFromStrError,
    Nix32HashStr,
};

/// A parsed NAR file name of the form `{file_hash}.nar` or
/// `{file_hash}.nar.{ext}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarFileName {
    file_hash: Nix32HashStr<32>,
    ext: Option<CompressionExtension>,
}

/// The type of error that can occur when parsing a string into a
/// [`NarFileName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NarFileNameFromStrError {
    /// The compression extension after `.nar.` is invalid.
    #[display("{_0}")]
    InvalidCompressionExtension(CompressionExtensionFromStrError),

    /// The file hash part is invalid.
    #[display("{_0}")]
    InvalidFileHash(Nix32HashFromStrError<32>),

    /// The string doesn't end with `.nar` or `.nar.{ext}`.
    #[display("missing '.nar' extension")]
    MissingNarExtension,
}

impl FromStr for NarFileName {
    type Err = NarFileNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to match `{hash}.nar.{ext}` first, then `{hash}.nar`.
        let (hash_str, ext) = if let Some((before, ext_str)) =
            s.strip_suffix(".nar").map(|h| (h, None)).or_else(|| {
                let (rest, ext_str) = s.rsplit_once('.')?;
                let hash_str = rest.strip_suffix(".nar")?;
                Some((hash_str, Some(ext_str)))
            }) {
            let ext = ext_str
                .map(|e| {
                    e.parse::<CompressionExtension>().map_err(
                        NarFileNameFromStrError::InvalidCompressionExtension,
                    )
                })
                .transpose()?;
            (before, ext)
        } else {
            return Err(NarFileNameFromStrError::MissingNarExtension);
        };

        let file_hash = hash_str
            .parse::<Nix32HashStr<32>>()
            .map_err(NarFileNameFromStrError::InvalidFileHash)?;

        Ok(Self { file_hash, ext })
    }
}

impl<'de> serde::de::Deserialize<'de> for NarFileName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
