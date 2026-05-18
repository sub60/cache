use alloc::borrow::Cow;
use core::fmt::{self, Display};
use core::str::FromStr;

use crate::{
    NixStoreLiteral,
    StoreBaseNameFromStrError,
    StoreBasename,
    StoreHash,
};

/// The type representing a Nix store path.
///
/// It consists of a store directory (usually `"/nix/store"`), a 32-character
/// hash, and a name.
#[derive(Clone, PartialEq, Eq)]
pub struct StorePath<StoreDir = NixStoreLiteral> {
    store_dir: StoreDir,
    basename: StoreBasename,
}

/// The type of error that can occur when parsing a string into a [`StorePath`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum StorePathFromStrError<StoreDir> {
    /// The path's store directory is invalid.
    StoreDir(StoreDir),

    /// The path's basename is invalid.
    BaseName(StoreBaseNameFromStrError),

    /// The string is missing a forward slash (`/`) after the store directory.
    MissingSlashAfterStoreDir,
}

impl StorePath {
    /// Creates a new [`StorePath`] with the given hash and name.
    pub fn new(basename: StoreBasename) -> Self {
        Self { store_dir: NixStoreLiteral, basename }
    }
}

impl<StoreDir> StorePath<StoreDir> {
    /// Returns the basename of this [`StorePath`].
    pub fn basename(&self) -> &StoreBasename {
        &self.basename
    }

    /// Returns the hash component of this [`StorePath`].
    pub fn hash(&self) -> &StoreHash {
        &self.basename.hash
    }

    /// Returns the hash component of this [`StorePath`], consuming `self`.
    pub fn into_hash(self) -> StoreHash {
        self.basename.hash
    }

    /// Returns the store directory component of this [`StorePath`].
    pub fn store_dir(&self) -> &StoreDir {
        &self.store_dir
    }

    /// Converts this [`StorePath`] to a new one with the given store
    /// directory.
    ///
    /// This together with [`StorePath::new`] allows to create a [`StorePath`]
    /// with different store directory than `"/nix/store"`.
    pub fn with_store_dir<NewDir>(self, new_dir: NewDir) -> StorePath<NewDir> {
        StorePath { store_dir: new_dir, basename: self.basename }
    }

    pub(crate) fn fmt_display(
        store_dir: &StoreDir,
        basename: &StoreBasename,
        f: &mut fmt::Formatter,
    ) -> fmt::Result
    where
        StoreDir: fmt::Display,
    {
        write!(f, "{}/{}", store_dir, basename)
    }
}

impl<StoreDir: Display> fmt::Debug for StorePath<StoreDir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<StoreDir: Display> fmt::Display for StorePath<StoreDir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Self::fmt_display(&self.store_dir, &self.basename, f)
    }
}

impl<StoreDir: FromStr> FromStr for StorePath<StoreDir> {
    type Err = StorePathFromStrError<StoreDir::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(store_dir_len) = memchr::memrchr(b'/', s.as_bytes()) else {
            return Err(StorePathFromStrError::MissingSlashAfterStoreDir);
        };

        let (store_dir_str, slash_and_basename) = s.split_at(store_dir_len);

        let store_dir =
            store_dir_str.parse().map_err(StorePathFromStrError::StoreDir)?;

        let basename = slash_and_basename[1..]
            .parse()
            .map_err(StorePathFromStrError::BaseName)?;

        Ok(Self { store_dir, basename })
    }
}

impl<'de, StoreDir> serde::de::Deserialize<'de> for StorePath<StoreDir>
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
