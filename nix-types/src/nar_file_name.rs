use alloc::borrow::Cow;
use core::fmt;
use core::str::FromStr;

use crate::{
    CompressionExtension,
    CompressionExtensionFromStrError,
    NarInfo,
    Nix32Digest,
    Nix32DigestFromStrError,
};

/// A parsed NAR file name of the form `{file_hash}.nar` or
/// `{file_hash}.nar.{ext}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NarFileName {
    /// The SHA-256 hash of the (possibly compressed) NAR file.
    pub file_hash: Nix32Digest<32>,
    /// The extension of the NAR file, if any.
    pub extension: Option<CompressionExtension>,
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
    InvalidFileHash(#[from] Nix32DigestFromStrError<32>),

    /// The string doesn't end with `.nar` or `.nar.{ext}`.
    #[display("missing '.nar' extension")]
    MissingNarExtension,
}

impl fmt::Display for NarFileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.nar", self.file_hash.as_str())?;
        if let Some(ext) = self.extension {
            write!(f, ".{ext}")?;
        }
        Ok(())
    }
}

impl FromStr for NarFileName {
    type Err = NarFileNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < Nix32Digest::<32>::NUM_CHARACTERS {
            return Err(Nix32DigestFromStrError::InvalidLength.into());
        }

        let (file_hash, rest) = s.split_at(Nix32Digest::<32>::NUM_CHARACTERS);

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

        Ok(Self { file_hash: file_hash.parse()?, extension: compression_ext })
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

impl<NarUrl> From<&NarInfo<NarUrl>> for NarFileName {
    fn from(narinfo: &NarInfo<NarUrl>) -> Self {
        Self {
            file_hash: narinfo.file_hash,
            extension: narinfo.compression.extension(),
        }
    }
}
