use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

use crate::types::{
    NixStoreHash,
    NixStoreHashFromStrError,
    NixStoreName,
    NixStoreNameFromStrError,
};

/// TODO: docs.
#[derive(Clone, PartialEq, Eq)]
pub struct NixStoreBaseName {
    /// TODO: docs.
    pub hash: NixStoreHash,

    /// TODO: docs.
    pub name: NixStoreName,
}

/// The type of error that can occur when parsing a string into a
/// [`NixStoreBaseName`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum NixStoreBaseNameFromStrError {
    /// The hash is invalid.
    Hash(NixStoreHashFromStrError),

    /// The name is invalid.
    Name(NixStoreNameFromStrError),

    /// The string is missing a dash (`-`) after the store hash.
    MissingDashAfterHash,
}

impl fmt::Debug for NixStoreBaseName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl fmt::Display for NixStoreBaseName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.hash, self.name)
    }
}

impl FromStr for NixStoreBaseName {
    type Err = NixStoreBaseNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < NixStoreHash::NUM_CHARACTERS {
            return Err(NixStoreBaseNameFromStrError::Hash(
                NixStoreHashFromStrError::InvalidLength,
            ));
        }

        let (hash_str, dash_and_name) =
            s.split_at(NixStoreHash::NUM_CHARACTERS);

        let hash =
            hash_str.parse().map_err(NixStoreBaseNameFromStrError::Hash)?;

        if !dash_and_name.starts_with('-') {
            return Err(NixStoreBaseNameFromStrError::MissingDashAfterHash);
        }

        let name = dash_and_name[1..]
            .parse()
            .map_err(NixStoreBaseNameFromStrError::Name)?;

        Ok(Self { hash, name })
    }
}

impl<'de> serde::de::Deserialize<'de> for NixStoreBaseName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
