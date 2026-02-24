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

/// TODO: docs.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub struct ContentAddressFromStrError;

impl ContentAddressMethod {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Fixed => "fixed",
            Self::FixedRecursive => "fixed:r",
            Self::FixedGit => "fixed:git",
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

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl fmt::Display for ContentAddressMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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
