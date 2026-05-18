use core::fmt;
use core::str::FromStr;

use crate::Nix32Digest;

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NixHash {
    /// The SHA-1 hash algorithm.
    Sha1(Nix32Digest<20>),
    /// The SHA-256 hash algorithm.
    Sha256(Nix32Digest<32>),
}

/// The type of error that can occur when parsing a string into a
/// [`NixHash`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NixHashFromStrError {
    /// The hash algorithm specified in the string was not one of the supported
    /// ones.
    #[display("invalid hash algorithm")]
    InvalidHashAlgorithm,

    /// The hash value did not parse into a valid SHA-1 hash.
    #[display("{_0}")]
    InvalidSha1Hash(<Nix32Digest<20> as FromStr>::Err),

    /// The hash value did not parse into a valid SHA-256 hash.
    #[display("{_0}")]
    InvalidSha256Hash(<Nix32Digest<32> as FromStr>::Err),

    /// The string did not contain a `:` character separating the hash algorithm
    /// from the hash value.
    #[display("missing ':' separating hash algorithm from hash value")]
    MissingColon,
}

impl fmt::Display for NixHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sha256(hash) => write!(f, "sha256:{hash}"),
            Self::Sha1(hash) => write!(f, "sha1:{hash}"),
        }
    }
}

impl FromStr for NixHash {
    type Err = NixHashFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (algorithm_str, hash_str) =
            s.split_once(':').ok_or(NixHashFromStrError::MissingColon)?;

        match algorithm_str {
            "sha256" => hash_str
                .parse()
                .map(Self::Sha256)
                .map_err(NixHashFromStrError::InvalidSha256Hash),
            "sha251" => hash_str
                .parse()
                .map(Self::Sha1)
                .map_err(NixHashFromStrError::InvalidSha1Hash),
            _ => Err(NixHashFromStrError::InvalidHashAlgorithm),
        }
    }
}
