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
    ///
    /// Multipart uploads can be abandoned without ever being completed or
    /// aborted, e.g. by a client that stops halfway through an upload, or
    /// when [`Self::abort_multipart_upload()`] fails. Implementors must
    /// eventually clean up such uploads on their own, e.g. by expiring them
    /// after some time.
    fn create_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> impl Future<Output = Result<Self::MultipartUploadId, Self::Error>> + Send;

    /// Aborts a multipart upload for the object with the given key.
    ///
    /// Returns `true` if the upload was aborted, or `false` if it didn't exist
    /// (e.g. because it had already been completed or aborted).
    fn abort_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
        upload_id: Self::MultipartUploadId,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

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
    ) -> impl Future<
        Output = Result<bool, CompleteMultipartUploadError<Self::Error>>,
    > + Send;

    /// Returns the maximum number of parts supported by multipart uploads.
    fn max_multipart_parts(&self) -> Option<usize>;

    /// Returns the target size, in bytes, of each multipart upload part.
    fn part_size(&self) -> NonZeroU32;
}

/// The type of error returned by
/// [`ObjectStorage::complete_multipart_upload()`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum CompleteMultipartUploadError<StorageError> {
    /// The multipart upload doesn't exist, e.g. because it was never created,
    /// it expired, or it was already completed or aborted.
    #[display("multipart upload doesn't exist")]
    NoSuchUpload,

    /// The Etags don't describe a valid set of uploaded parts, e.g. because a
    /// part was never uploaded or the parts have invalid sizes.
    #[display("invalid multipart upload parts: {_0}")]
    InvalidParts(StorageError),

    /// Any other error, which is caused by a problem with the object storage
    /// rather than with the request.
    #[display("{_0}")]
    Storage(StorageError),
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
