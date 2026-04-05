use core::fmt::{self, Display};
use core::str::FromStr;
use std::borrow::Cow;

use crate::{
    NixStoreBaseName,
    NixStoreBaseNameFromStrError,
    NixStoreHash,
    NixStoreLiteral,
};

/// The type representing a Nix store path.
///
/// It consists of a store directory (usually `"/nix/store"`), a 32-character
/// hash, and a name.
#[derive(Clone, PartialEq, Eq)]
pub struct NixStorePath<StoreDir = NixStoreLiteral> {
    store_dir: StoreDir,
    basename: NixStoreBaseName,
}

/// The type of error that can occur when parsing a string into a
/// [`NixStorePath`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum NixStorePathFromStrError<StoreDir> {
    /// The path's store directory is invalid.
    StoreDir(StoreDir),

    /// The path's basename is invalid.
    BaseName(NixStoreBaseNameFromStrError),

    /// The string is missing a forward slash (`/`) after the store directory.
    MissingSlashAfterStoreDir,
}

impl NixStorePath {
    /// Creates a new [`NixStorePath`] with the given hash and name.
    pub fn new(basename: NixStoreBaseName) -> Self {
        Self { store_dir: NixStoreLiteral, basename }
    }
}

impl<StoreDir> NixStorePath<StoreDir> {
    /// Returns the basename of this [`NixStorePath`].
    pub fn basename(&self) -> &NixStoreBaseName {
        &self.basename
    }

    /// Returns the hash component of this [`NixStorePath`].
    pub fn hash(&self) -> &NixStoreHash {
        &self.basename.hash
    }

    /// Returns the hash component of this [`NixStorePath`], consuming `self`.
    pub fn into_hash(self) -> NixStoreHash {
        self.basename.hash
    }

    /// Returns the store directory component of this [`NixStorePath`].
    pub fn store_dir(&self) -> &StoreDir {
        &self.store_dir
    }

    /// Converts this [`NixStorePath`] to a new one with the given store
    /// directory.
    ///
    /// This together with [`NixStorePath::new`] allows to create a
    /// [`NixStorePath`] with different store directory than `"/nix/store"`.
    pub fn with_store_dir<NewDir>(
        self,
        new_dir: NewDir,
    ) -> NixStorePath<NewDir> {
        NixStorePath { store_dir: new_dir, basename: self.basename }
    }

    pub(crate) fn fmt_display(
        store_dir: &StoreDir,
        basename: &NixStoreBaseName,
        f: &mut fmt::Formatter,
    ) -> fmt::Result
    where
        StoreDir: fmt::Display,
    {
        write!(f, "{}/{}", store_dir, basename)
    }
}

impl<StoreDir: Display> fmt::Debug for NixStorePath<StoreDir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<StoreDir: Display> fmt::Display for NixStorePath<StoreDir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::fmt_display(&self.store_dir, &self.basename, f)
    }
}

impl<StoreDir: FromStr> FromStr for NixStorePath<StoreDir> {
    type Err = NixStorePathFromStrError<StoreDir::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(store_dir_len) = memchr::memrchr(b'/', s.as_bytes()) else {
            return Err(NixStorePathFromStrError::MissingSlashAfterStoreDir);
        };

        let (store_dir_str, slash_and_basename) = s.split_at(store_dir_len);

        let store_dir = store_dir_str
            .parse()
            .map_err(NixStorePathFromStrError::StoreDir)?;

        let basename = slash_and_basename[1..]
            .parse()
            .map_err(NixStorePathFromStrError::BaseName)?;

        Ok(Self { store_dir, basename })
    }
}

impl<'de, StoreDir> serde::de::Deserialize<'de> for NixStorePath<StoreDir>
where
    StoreDir: FromStr<Err: fmt::Display>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
