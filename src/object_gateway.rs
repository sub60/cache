//! TODO: docs.

use core::num::NonZeroU16;
use core::time::Duration;

use axum::http;

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;

/// TODO: docs.
pub trait ObjectGateway: Sync {
    /// TODO: docs.
    type Storage: ObjectStorage;

    /// Returns a URL which is able to fulfill the given access request to an
    /// object in the storage.
    fn object_url(&self, request: ObjectUrlRequest<'_>) -> impl AsRef<str>;

    /// Returns a URL which can be used to upload a multipart upload part.
    fn multipart_upload_url(
        &self,
        request: MultipartUploadUrlRequest<
            '_,
            <Self::Storage as ObjectStorage>::MultipartUploadId,
        >,
    ) -> impl AsRef<str>;

    /// TODO: docs.
    fn storage(&self) -> &Self::Storage;
}

/// TODO: docs.
pub struct ObjectUrlRequest<'a> {
    /// The key of the object for which to generate a presigned URL.
    pub key: ObjectStorageKey<'a>,
    /// The HTTP method that the presigned URL should allow.
    pub method: http::Method,
    /// The TTL for the presigned URL.
    pub min_ttl: Duration,
}

/// TODO: docs.
pub struct MultipartUploadUrlRequest<'a, UploadId> {
    /// The key of the object for which to generate a presigned URL.
    pub key: ObjectStorageKey<'a>,
    /// TODO: docs.
    pub part_number: NonZeroU16,
    /// TODO: docs.
    pub upload_id: &'a UploadId,
    /// The TTL for the presigned URL.
    pub min_ttl: Duration,
}
