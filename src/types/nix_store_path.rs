use core::fmt::{self, Display};
use core::str::FromStr;
use std::borrow::Cow;

use crate::types::{
    NixStoreBaseName,
    NixStoreBaseNameFromStrError,
    NixStoreLiteral,
};

/// The type representing a Nix store path.
///
/// It consists of a prefix (usually `"/nix/store"`), a 32-character hash, and a
/// name.
#[derive(Clone, PartialEq, Eq)]
pub struct NixStorePath<StorePrefix = NixStoreLiteral> {
    prefix: StorePrefix,
    basename: NixStoreBaseName,
}

/// The type of error that can occur when parsing a string into a
/// [`NixStorePath`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum NixStorePathFromStrError<StorePrefix> {
    /// The path's prefix is invalid.
    Prefix(StorePrefix),

    /// The path's basename is invalid.
    BaseName(NixStoreBaseNameFromStrError),

    /// The string is missing a forward slash (`/`) after the prefix.
    MissingSlashAfterPrefix,
}

impl NixStorePath {
    /// Creates a new [`NixStorePath`] with the given hash and name.
    pub fn new(basename: NixStoreBaseName) -> Self {
        Self { prefix: NixStoreLiteral, basename }
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
        NixStorePath { prefix: new_prefix, basename: self.basename }
    }
}

impl<StorePrefix: Display> fmt::Debug for NixStorePath<StorePrefix> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<StorePrefix: Display> fmt::Display for NixStorePath<StorePrefix> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.prefix, self.basename)
    }
}

impl<StorePrefix: FromStr> FromStr for NixStorePath<StorePrefix> {
    type Err = NixStorePathFromStrError<StorePrefix::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(prefix_len) = memchr::memrchr(b'/', s.as_bytes()) else {
            return Err(NixStorePathFromStrError::MissingSlashAfterPrefix);
        };

        let (prefix_str, slash_and_basename) = s.split_at(prefix_len);

        let prefix =
            prefix_str.parse().map_err(NixStorePathFromStrError::Prefix)?;

        let basename = slash_and_basename[1..]
            .parse()
            .map_err(NixStorePathFromStrError::BaseName)?;

        Ok(Self { prefix, basename })
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
