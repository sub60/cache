//! TODO: docs.

mod in_memory;

use core::error::Error;
use core::num::NonZeroU32;

pub use in_memory::{InMemoryObjectStorage, InMemoryObjectStorageError};
use nix_types::sub60::nar_multiparts::ETag;
use nix_types::{NarFileName, NarInfoFileName, StoreBasename};
use serde::de::DeserializeOwned;

/// TODO: docs.
pub trait ObjectStorage: Sync {
    /// TODO: docs.
    type MultipartUploadId: Into<String> + DeserializeOwned + Send;

    /// TODO: docs.
    type Error: Error;

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

    /// Moves an object from one key to another.
    ///
    /// If an object already exists under the destination key, it should be
    /// overwritten.
    fn r#move<'src, 'dst>(
        &self,
        source: impl Into<ObjectStorageKey<'src>> + Send,
        destination: impl Into<ObjectStorageKey<'dst>> + Send,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

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

    /// Completes a multipart upload for the object with the given key.
    ///
    /// Implementors can assume `etags.len()` is less than or equal to
    /// [`Self::max_multipart_parts()`], when a maximum is returned.
    fn complete_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
        upload_id: Self::MultipartUploadId,
        etags: &[ETag],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Returns the maximum number of parts supported by multipart uploads.
    fn max_multipart_parts(&self) -> Option<usize>;

    /// Returns the target size, in bytes, of each multipart upload part.
    fn part_size(&self) -> NonZeroU32;
}

/// TODO: docs..
#[derive(Debug, Copy, Clone)]
pub enum ObjectStorageKey<'a> {
    /// TODO: docs.
    Nar(&'a NarFileName),

    /// TODO: docs.
    NarInfo(&'a NarInfoFileName),

    /// TODO: docs.
    NarMultipart(&'a StoreBasename),
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

impl<'a> From<&'a StoreBasename> for ObjectStorageKey<'a> {
    fn from(store_basename: &'a StoreBasename) -> Self {
        Self::NarMultipart(store_basename)
    }
}
