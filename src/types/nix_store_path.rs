use core::fmt::{self, Display};
use core::str::FromStr;
use std::borrow::Cow;

use crate::types::{
    NixStoreHash,
    NixStoreHashFromStrError,
    NixStoreLiteral,
    NixStoreName,
    NixStoreNameFromStrError,
};

/// The type representing a Nix store path.
///
/// It consists of a prefix (usually `"/nix/store"`), a 32-character hash, and a
/// name.
#[derive(Clone, PartialEq, Eq)]
pub struct NixStorePath<StorePrefix = NixStoreLiteral> {
    prefix: StorePrefix,
    hash: NixStoreHash,
    name: NixStoreName,
}

/// The type of error that can occur when parsing a string into a
/// [`NixStorePath`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum NixStorePathFromStrError<StorePrefix> {
    /// The path's prefix is invalid.
    Prefix(StorePrefix),

    /// The path's hash is invalid.
    Hash(NixStoreHashFromStrError),

    /// The path's name is invalid.
    Name(NixStoreNameFromStrError),

    /// The string is missing a forward slash (`/`) after the prefix.
    MissingSlashAfterPrefix,

    /// The string is missing a dash (`-`) after the store hash.
    MissingDashAfterHash,
}

impl NixStorePath {
    /// Creates a new [`NixStorePath`] with the given hash and name.
    pub fn new(hash: NixStoreHash, name: NixStoreName) -> Self {
        Self { prefix: NixStoreLiteral, hash, name }
    }
}

impl<StorePrefix> NixStorePath<StorePrefix> {
    /// Converts this [`NixStorePath`] to a new one with the given store prefix.
    ///
    /// This together with [`NixStorePath::new`] allows to create a
    /// [`NixStorePath`] with different store prefix than `"/nix/store"`.
    pub fn with_prefix<NewPrefix>(
        self,
        new_prefix: NewPrefix,
    ) -> NixStorePath<NewPrefix> {
        NixStorePath { prefix: new_prefix, hash: self.hash, name: self.name }
    }
}

impl<StorePrefix: Display> fmt::Debug for NixStorePath<StorePrefix> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<StorePrefix: Display> fmt::Display for NixStorePath<StorePrefix> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}-{}", self.prefix, self.hash, self.name)
    }
}

impl<StorePrefix: FromStr> FromStr for NixStorePath<StorePrefix> {
    type Err = NixStorePathFromStrError<StorePrefix::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(prefix_len) = memchr::memrchr(b'/', s.as_bytes()) else {
            return Err(NixStorePathFromStrError::MissingSlashAfterPrefix);
        };

        let (prefix_str, rest) = s.split_at(prefix_len);

        let prefix =
            prefix_str.parse().map_err(NixStorePathFromStrError::Prefix)?;

        if rest.len() < 1 + NixStoreHash::NUM_CHARACTERS {
            return Err(NixStorePathFromStrError::Hash(
                NixStoreHashFromStrError::InvalidLength,
            ));
        }

        let (hash_str, dash_and_name) =
            rest[1..].split_at(NixStoreHash::NUM_CHARACTERS);

        let hash = hash_str.parse().map_err(NixStorePathFromStrError::Hash)?;

        if !dash_and_name.starts_with('-') {
            return Err(NixStorePathFromStrError::MissingDashAfterHash);
        }

        let name = dash_and_name[1..]
            .parse()
            .map_err(NixStorePathFromStrError::Name)?;

        Ok(Self { prefix, hash, name })
    }
}

impl<'de, StorePrefix> serde::de::Deserialize<'de> for NixStorePath<StorePrefix>
where
    StorePrefix: FromStr<Err: fmt::Display>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse::<Self>()
            .map_err(serde::de::Error::custom)
    }
}
