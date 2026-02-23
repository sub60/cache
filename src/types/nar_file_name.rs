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
#[derive(
    Debug, PartialEq, Eq, derive_more::Display, cauchy::Error, cauchy::From,
)]
pub enum NarFileNameFromStrError {
    /// The character immediately following the `.nar` extension is invalid
    /// (i.e. not a dot).
    #[display("invalid character '{_0}' following '.nar' extension")]
    InvalidCharacterAfterNarExtension(char),

    /// The compression extension after `.nar.` is invalid.
    #[display("{_0}")]
    InvalidCompressionExtension(#[from] CompressionExtensionFromStrError),

    /// The file hash part is invalid.
    #[display("{_0}")]
    InvalidFileHash(#[from] Nix32HashFromStrError<32>),

    /// The string doesn't end with `.nar` or `.nar.{ext}`.
    #[display("missing '.nar' extension")]
    MissingNarExtension,
}

impl FromStr for NarFileName {
    type Err = NarFileNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < Nix32HashStr::<32>::NUM_CHARACTERS {
            return Err(Nix32HashFromStrError::InvalidLength.into());
        }

        let (file_hash, rest) = s.split_at(Nix32HashStr::<32>::NUM_CHARACTERS);

        let Some(maybe_dot_and_ext) = rest.strip_prefix(".nar") else {
            return Err(NarFileNameFromStrError::MissingNarExtension);
        };

        let compression_ext =
            match maybe_dot_and_ext.chars().next() {
                Some('.') => maybe_dot_and_ext[1..].parse().map(Some)?,
                Some(other) => return Err(
                    NarFileNameFromStrError::InvalidCharacterAfterNarExtension(
                        other,
                    ),
                ),
                None => None,
            };

        Ok(Self { file_hash: file_hash.parse()?, ext: compression_ext })
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
