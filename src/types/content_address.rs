use core::fmt;
use core::str::FromStr;

use crate::types::Nix32Hash;

/// TODO: docs.
#[derive(Debug)]
pub struct ContentAddress {
    method: ContentAddressMethod,
    hash: ContentAddressHash,
}

/// TODO: docs.
#[derive(Debug, Copy, Clone)]
pub enum ContentAddressMethod {
    /// TODO: docs.
    Text,
    /// TODO: docs.
    Fixed,
    /// TODO: docs.
    FixedRecursive,
    /// TODO: docs.
    FixedGit,
}

/// TODO: docs.
#[derive(Debug, Copy, Clone)]
pub enum ContentAddressHash {
    /// The SHA-1 hash algorithm.
    Sha1(Nix32Hash<20>),
    /// The SHA-256 hash algorithm.
    Sha256(Nix32Hash<32>),
}

/// The type of error that can occur when parsing a string into a
/// [`ContentAddress`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum ContentAddressFromStrError {
    /// The string did not contain a `:` character separating the method from
    /// the hash.
    #[display("invalid hash: {_0}")]
    Hash(<ContentAddressHash as FromStr>::Err),

    /// The string did not contain a `:` character separating the method from
    /// the hash.
    #[display("invalid method: {_0}")]
    Method(<ContentAddressMethod as FromStr>::Err),

    /// The string did not contain a `:` character separating the method from
    /// the hash.
    #[display("missing ':' separating method from hash")]
    MissingColon,
}

/// The type of error that can occur when parsing a string into a
/// [`ContentAddressMethod`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum ContentAddressMethodFromStrError {
    #[display("invalid content address method")]
    InvalidMethod,
}

/// The type of error that can occur when parsing a string into a
/// [`ContentAddressMethod`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum ContentAddressHashFromStrError {
    /// The hash algorithm specified in the string was not one of the supported
    /// ones.
    #[display("invalid hash algorithm")]
    InvalidHashAlgorithm,

    /// The hash value did not parse into a valid SHA-1 hash.
    #[display("{_0}")]
    InvalidSha1Hash(<Nix32Hash<20> as FromStr>::Err),

    /// The hash value did not parse into a valid SHA-256 hash.
    #[display("{_0}")]
    InvalidSha256Hash(<Nix32Hash<32> as FromStr>::Err),

    /// The string did not contain a `:` character separating the hash algorithm
    /// from the hash value.
    #[display("missing ':' separating hash algorithm from hash value")]
    MissingColon,
}

impl ContentAddressMethod {
    const TEXT_STR: &'static str = "text";
    const FIXED_STR: &'static str = "fixed";
    const FIXED_RECURSIVE_STR: &'static str = "fixed:r";
    const FIXED_GIT_STR: &'static str = "fixed:git";

    const fn as_str(self) -> &'static str {
        match self {
            Self::Text => Self::TEXT_STR,
            Self::Fixed => Self::FIXED_STR,
            Self::FixedRecursive => Self::FIXED_RECURSIVE_STR,
            Self::FixedGit => Self::FIXED_GIT_STR,
        }
    }
}

impl fmt::Display for ContentAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.method, self.hash)
    }
}

impl FromStr for ContentAddress {
    type Err = ContentAddressFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (method_str, hash_str) = s
            .split_once(':')
            .ok_or(ContentAddressFromStrError::MissingColon)?;

        let method =
            method_str.parse().map_err(ContentAddressFromStrError::Method)?;

        let hash =
            hash_str.parse().map_err(ContentAddressFromStrError::Hash)?;

        Ok(Self { method, hash })
    }
}

impl fmt::Display for ContentAddressMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ContentAddressMethod {
    type Err = ContentAddressMethodFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::TEXT_STR => Ok(Self::Text),
            Self::FIXED_STR => Ok(Self::Fixed),
            Self::FIXED_RECURSIVE_STR => Ok(Self::FixedRecursive),
            Self::FIXED_GIT_STR => Ok(Self::FixedGit),
            _ => Err(ContentAddressMethodFromStrError::InvalidMethod),
        }
    }
}

impl fmt::Display for ContentAddressHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sha1(hash) => write!(f, "sha1:{hash}"),
            Self::Sha256(hash) => write!(f, "sha256:{hash}"),
        }
    }
}

impl FromStr for ContentAddressHash {
    type Err = ContentAddressHashFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (algorithm_str, hash_str) = s
            .split_once(':')
            .ok_or(ContentAddressHashFromStrError::MissingColon)?;

        match algorithm_str {
            "sha251" => hash_str
                .parse()
                .map(Self::Sha1)
                .map_err(ContentAddressHashFromStrError::InvalidSha1Hash),
            "sha256" => hash_str
                .parse()
                .map(Self::Sha256)
                .map_err(ContentAddressHashFromStrError::InvalidSha256Hash),
            _ => Err(ContentAddressHashFromStrError::InvalidHashAlgorithm),
        }
    }
}
