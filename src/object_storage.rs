//! TODO: docs.

use core::error::Error;
use core::fmt;
use core::num::NonZeroU32;

use nix_types::{NarFileName, NarInfoFileName};
use serde::de::DeserializeOwned;

use crate::CacheId;
use crate::nar_multiparts::Etag;

/// TODO: docs.
pub trait ObjectStorage: Sync {
    /// TODO: docs.
    type MultipartUploadId: Clone + fmt::Display + DeserializeOwned + Send;

    /// TODO: docs.
    type Error: Error + Send;

    /// TODO: docs.
    fn get(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> impl Future<
        Output = Result<Option<impl AsRef<[u8]> + Send>, Self::Error>,
    > + Send;

    /// Inserts an object if no object with the same key already exists.
    ///
    /// Returns `true` if the object was inserted, or `false` if an object with
    /// the same key already existed.
    fn insert(
        &self,
        key: ObjectStorageKey<'_>,
        bytes: impl AsRef<[u8]> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Returns whether the object storage contains the given key.
    fn contains_key(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Creates a new multipart upload ID for an object with the given key.
    fn create_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> impl Future<Output = Result<Self::MultipartUploadId, Self::Error>> + Send;

    /// Aborts a multipart upload for the object with the given key.
    fn abort_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
        upload_id: Self::MultipartUploadId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Completes a multipart upload for the object with the given key.
    ///
    /// Returns `true` if the object was inserted, or `false` if an object with
    /// the same key already existed.
    ///
    /// Implementors can assume `etags.len()` is less than or equal to
    /// [`Self::max_multipart_parts()`], when a maximum is returned.
    fn complete_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
        upload_id: Self::MultipartUploadId,
        etags: &[Etag],
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

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

    /// A NARInfo of a given cache.
    ///
    /// This variant includes the cache ID because unlike NARs, which are
    /// content-addressed and shared by all caches, NARInfos are scoped to a
    /// single cache.
    NarInfo(&'a CacheId, &'a NarInfoFileName),
}
