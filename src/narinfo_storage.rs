//! TODO: docs.

use core::error::Error;
use core::fmt;

use nix_types::{NarInfo, NarInfoFileName};

use crate::CacheInfos;

/// The storage of the caches' NARInfos.
pub trait NarInfoStorage: Sync {
    /// The type of error returned by the storage.
    type Error: Error + Send;

    /// Returns whether the given NARInfo of the given cache is stored.
    fn contains(
        &self,
        cache: &CacheInfos,
        narinfo: &NarInfoFileName,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Returns the given NARInfo of the given cache, if it's stored.
    fn get(
        &self,
        cache: &CacheInfos,
        narinfo: &NarInfoFileName,
    ) -> impl Future<
        Output = Result<Option<impl AsRef<[u8]> + Send>, Self::Error>,
    > + Send;

    /// Inserts a NARInfo into the given cache under the given file name, unless
    /// the cache already has one with the same file name.
    ///
    /// Once inserted, [`Self::get()`] will be expected to return a byte slice
    /// matching the given NARInfo's [`Display`](fmt::Display) representation.
    ///
    /// Returns `true` if the NARInfo was inserted, or `false` if the cache
    /// already had one.
    fn insert(
        &self,
        cache: &CacheInfos,
        file_name: &NarInfoFileName,
        narinfo: &NarInfo<impl fmt::Display + Sync>,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}
