//! TODO: docs.

mod in_memory;

use core::error::Error;

pub use in_memory::InMemoryObjectStorage;
use types::{NarFileName, NarInfoFileName};

/// TODO: docs.
pub trait ObjectStorage: Sync {
    /// TODO: docs.
    type Error: Error + Send;

    /// Returns whether the object storage contains the given key.
    fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// TODO: docs.
    fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
    ) -> impl Future<Output = Result<Option<impl AsRef<[u8]>>, Self::Error>> + Send;

    /// TODO: docs.
    fn set<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
        bytes: impl AsRef<[u8]> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}

/// TODO: docs..
#[derive(Debug, Copy, Clone)]
pub enum ObjectStorageKey<'a> {
    /// TODO: docs..
    Nar(&'a NarFileName),
    /// TODO: docs..
    NarInfo(&'a NarInfoFileName),
}

impl<'a> From<&'a NarFileName> for ObjectStorageKey<'a> {
    fn from(nar_file_name: &'a NarFileName) -> Self {
        Self::Nar(nar_file_name)
    }
}

impl<'a> From<&'a NarInfoFileName> for ObjectStorageKey<'a> {
    fn from(narinfo_file_name: &'a NarInfoFileName) -> Self {
        Self::NarInfo(narinfo_file_name)
    }
}
