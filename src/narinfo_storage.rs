//! TODO: docs.

use core::error::Error;

use nix_types::NarInfoFileName;

use crate::CacheId;

/// The storage of the caches' NARInfos.
pub trait NarInfoStorage: Sync {
    /// The type of error returned by the storage.
    type Error: Error + Send;

    /// Returns the given NARInfo of the given cache, if it's stored.
    fn get(
        &self,
        cache_id: &CacheId,
        narinfo: &NarInfoFileName,
    ) -> impl Future<
        Output = Result<Option<impl AsRef<[u8]> + Send>, Self::Error>,
    > + Send;

    /// Inserts a NARInfo into the given cache, unless the cache already has
    /// one with the same file name.
    ///
    /// Returns `true` if the NARInfo was inserted, or `false` if the cache
    /// already had one.
    fn insert(
        &self,
        cache_id: &CacheId,
        narinfo: &NarInfoFileName,
        bytes: impl AsRef<[u8]> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}
