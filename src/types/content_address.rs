use core::fmt;
use core::str::FromStr;

use crate::types::NixHash;

/// TODO: docs.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentAddress {
    method: ContentAddressMethod,
    hash: NixHash,
}

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// The type of error that can occur when parsing a string into a
/// [`ContentAddress`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum ContentAddressFromStrError {
    /// The hash did not parse into a valid [`NixHash`].
    #[display("invalid hash: {_0}")]
    Hash(<NixHash as FromStr>::Err),

    /// The method did not parse into a valid [`ContentAddressMethod`].
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
    /// The method specified in the string was invalid.
    #[display("invalid content address method")]
    InvalidMethod,
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
