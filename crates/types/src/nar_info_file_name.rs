use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

use crate::{NixStoreHash, NixStoreHashFromStrError};

/// A parsed NARInfo file name of the form `{store_hash}.narinfo`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NarInfoFileName {
    /// The store hash in the file name.
    pub store_hash: NixStoreHash,
}

/// The type of error that can occur when parsing a string into a
/// [`NarInfoFileName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NarInfoFileNameFromStrError {
    /// THe store hash part is invalid.
    #[display("{_0}")]
    InvalidStoreHash(NixStoreHashFromStrError),
    /// The string doesn't end with `.narinfo`.
    #[display("missing '.narinfo' extension")]
    MissingNarInfoExtension,
}

impl fmt::Display for NarInfoFileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.narinfo", self.store_hash.as_str())
    }
}

impl FromStr for NarInfoFileName {
    type Err = NarInfoFileNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_suffix(".narinfo")
            .ok_or(NarInfoFileNameFromStrError::MissingNarInfoExtension)?
            .parse()
            .map_err(NarInfoFileNameFromStrError::InvalidStoreHash)
            .map(|store_hash| Self { store_hash })
    }
}

impl<'de> serde::de::Deserialize<'de> for NarInfoFileName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
