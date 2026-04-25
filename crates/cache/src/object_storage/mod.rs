//! TODO: docs.

mod in_memory;

use core::error::Error;

pub use in_memory::InMemoryObjectStorage;
use nix_types::sub60::nar_multiparts;
use nix_types::{NarFileName, NarInfoFileName};

/// TODO: docs.
pub trait ObjectStorage: Sync {
    /// TODO: docs.
    type MultipartUploadId: Send;

    /// TODO: docs.
    type Error: Error;

    /// Returns whether the object storage contains the given key.
    fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Creates a new multipart upload ID for an object with the given key.
    fn create_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
    ) -> impl Future<Output = Result<Self::MultipartUploadId, Self::Error>> + Send;

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
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// TODO: docs..
#[derive(Debug, Copy, Clone)]
pub enum ObjectStorageKey<'a> {
    /// TODO: docs.
    Nar(&'a NarFileName),

    /// TODO: docs.
    NarMultipart(nar_multiparts::UploadId),

    /// TODO: docs.
    NarInfo(&'a NarInfoFileName),
}

impl<'a> From<&'a NarFileName> for ObjectStorageKey<'a> {
    fn from(nar_file_name: &'a NarFileName) -> Self {
        Self::Nar(nar_file_name)
    }
}

impl From<nar_multiparts::UploadId> for ObjectStorageKey<'_> {
    fn from(upload_id: nar_multiparts::UploadId) -> Self {
        Self::NarMultipart(upload_id)
    }
}

impl<'a> From<&'a NarInfoFileName> for ObjectStorageKey<'a> {
    fn from(narinfo_file_name: &'a NarInfoFileName) -> Self {
        Self::NarInfo(narinfo_file_name)
    }
}
