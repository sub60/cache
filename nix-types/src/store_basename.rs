use alloc::borrow::Cow;
use core::fmt;
use core::str::FromStr;

use crate::{
    StoreHash,
    StoreHashFromStrError,
    StoreName,
    StoreNameFromStrError,
};

/// TODO: docs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StoreBasename {
    /// TODO: docs.
    pub hash: StoreHash,

    /// TODO: docs.
    pub name: StoreName,
}

/// The type of error that can occur when parsing a string into a
/// [`StoreBasename`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum StoreBaseNameFromStrError {
    /// The hash is invalid.
    Hash(StoreHashFromStrError),

    /// The name is invalid.
    Name(StoreNameFromStrError),

    /// The string is missing a dash (`-`) after the store hash.
    MissingDashAfterHash,
}

impl fmt::Debug for StoreBasename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl fmt::Display for StoreBasename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.hash, self.name)
    }
}

impl FromStr for StoreBasename {
    type Err = StoreBaseNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < StoreHash::NUM_CHARACTERS {
            return Err(StoreBaseNameFromStrError::Hash(
                StoreHashFromStrError::InvalidLength,
            ));
        }

        let (hash_str, dash_and_name) = s.split_at(StoreHash::NUM_CHARACTERS);

        let hash = hash_str.parse().map_err(StoreBaseNameFromStrError::Hash)?;

        if !dash_and_name.starts_with('-') {
            return Err(StoreBaseNameFromStrError::MissingDashAfterHash);
        }

        let name = dash_and_name[1..]
            .parse()
            .map_err(StoreBaseNameFromStrError::Name)?;

        Ok(Self { hash, name })
    }
}

impl serde::ser::Serialize for StoreBasename {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::de::Deserialize<'de> for StoreBasename {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
