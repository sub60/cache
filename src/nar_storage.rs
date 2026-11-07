//! TODO: docs.

use core::error::Error;
use core::fmt;
use core::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use core::str::FromStr;
use core::time::Duration;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use nix_types::{NarFileName, Nix32Digest};

use crate::CacheInfos;
use crate::api::Etag;

/// The storage of NARs.
pub trait NarStorage: Sync {
    /// The ID of a multipart upload.
    type MultipartUploadId: Clone
        + fmt::Display
        + FromStr<Err: fmt::Display>
        + Send
        + Sync;

    /// The type of error returned by the storage.
    type Error: Error + Into<StatusCode> + Send;

    /// Completes the given multipart upload to the given cache.
    ///
    /// Implementors must compute the NAR's file hash from the bytes the upload
    /// stored, since clients never declare it. They can assume `etags.len()` is
    /// less than or equal to [`Self::max_multipart_parts()`], when a maximum is
    /// returned.
    fn complete_multipart_upload(
        &self,
        cache: &CacheInfos,
        upload_id: &Self::MultipartUploadId,
        etags: &[Etag],
    ) -> impl Future<
        Output = Result<
            CompletedMultipartUpload,
            CompleteMultipartUploadError<Self::Error>,
        >,
    > + Send;

    /// Returns whether the given NAR of the given cache is stored.
    fn contains(
        &self,
        cache: &CacheInfos,
        nar: &NarFileName,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Creates a new multipart upload of a NAR of `size` bytes to the given
    /// cache.
    ///
    /// Multipart uploads can be abandoned without ever being completed, e.g.
    /// by a client that stops halfway through an upload. Implementors must
    /// eventually clean up such uploads on their own, e.g. by expiring them
    /// after some time.
    fn create_multipart_upload(
        &self,
        cache: &CacheInfos,
        size: NonZeroU64,
    ) -> impl Future<Output = Result<Self::MultipartUploadId, Self::Error>> + Send;

    /// Returns the URL clients download the given NAR of the given cache from.
    ///
    /// It's the `URL` field of the NARInfos served to clients, so it can also
    /// be relative to the cache's URL. In particular, `nar/{nar}` points to
    /// the cache's NAR endpoint, which serves the NARs returned by
    /// [`Self::get()`].
    fn download_url(
        &self,
        cache: &CacheInfos,
        nar: &NarFileName,
    ) -> impl AsRef<str>;

    /// Returns the response with which the cache's NAR endpoint serves the
    /// given NAR of the given cache, or `None` if the NAR isn't stored.
    ///
    /// Implementors whose [`Self::download_url()`] never points to the NAR
    /// endpoint can always return `None`.
    fn get(
        &self,
        cache: &CacheInfos,
        nar: &NarFileName,
    ) -> impl Future<
        Output = Result<Option<impl IntoResponse + Send>, Self::Error>,
    > + Send;

    /// Stores the NAR in the body of the given request to the given cache.
    ///
    /// Returns `true` if the NAR was inserted, or `false` if it was already
    /// stored.
    ///
    /// Implementors must not store bodies SHA-256 hash isn't the NAR's
    /// [`file_hash`](NarFileName::file_hash), and should fail with a `400 Bad
    /// Request` error instead.
    fn insert(
        &self,
        cache: &CacheInfos,
        nar: &NarFileName,
        request: Request,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;

    /// Returns the maximum number of parts supported by multipart uploads.
    fn max_multipart_parts(&self) -> Option<usize>;

    /// Returns the URLs clients upload the given parts of the given multipart
    /// upload to the given cache to, in the same order as `part_numbers`, valid
    /// for at least `min_ttl`.
    fn multipart_upload_part_urls(
        &self,
        cache: &CacheInfos,
        upload_id: &Self::MultipartUploadId,
        part_numbers: &[NonZeroU16],
        min_ttl: Duration,
    ) -> impl Future<Output = Result<Vec<impl AsRef<str> + Send>, Self::Error>> + Send;

    /// Returns the target size, in bytes, of each multipart upload part.
    fn part_size(&self) -> NonZeroU32;
}

/// A multipart upload that completed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompletedMultipartUpload {
    /// The SHA-256 hash of the NAR the upload stored.
    pub file_hash: Nix32Digest<32>,

    /// Whether the upload inserted the NAR, which is `false` if a NAR with the
    /// same file hash was already stored.
    pub inserted: bool,
}

/// The type of error returned by
/// [`NarStorage::complete_multipart_upload()`].
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

    /// Any other error, which is caused by a problem with the storage rather
    /// than with the request.
    #[display("{_0}")]
    Storage(StorageError),
}
