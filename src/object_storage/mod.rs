//! TODO: docs.

mod r2;

use core::error::Error;

pub use r2::R2ObjectStorage;

use crate::types::{NarFileName, NixStoreHash};

/// TODO: docs.
pub trait ObjectStorage: Sync {
    /// TODO: docs.
    type Key<'a>: From<&'a NarFileName> + From<&'a NixStoreHash> + Copy + Send;

    /// TODO: docs.
    type Error: Error + Send;

    /// Returns whether the object storage contains the given key.
    fn contains_key(
        &self,
        key: Self::Key<'_>,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// TODO: docs.
    fn get(
        &self,
        key: Self::Key<'_>,
    ) -> impl Future<Output = Result<Option<impl AsRef<[u8]>>, Self::Error>> + Send;

    /// TODO: docs.
    fn set(
        &self,
        key: Self::Key<'_>,
        bytes: &[u8],
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}
