use core::num::{NonZeroU16, NonZeroU32};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use nix_types::{NarFileName, NarInfoFileName};
use sub60_cache::nar_multiparts::Etag;
use sub60_cache::{CacheId, ObjectStorage, ObjectStorageKey};

/// The ID of an in-memory multipart upload.
pub type MultipartUploadId = u64;

/// An [`ObjectStorage`] trait that stores objects and multipart sessions in
/// memory.
#[derive(Clone, Default)]
pub struct InMemoryObjectStorage(Arc<Mutex<InMemoryObjectStorageInner>>);

#[derive(Default)]
struct InMemoryObjectStorageInner {
    nars: HashMap<NarFileName, Bytes>,
    nar_infos: HashMap<(CacheId, NarInfoFileName), Bytes>,
    multipart_uploads: HashMap<MultipartUploadId, MultipartUpload>,
    next_upload_id: MultipartUploadId,
}

struct MultipartUpload {
    key: OwnedObjectStorageKey,
    parts: Vec<Option<Bytes>>,
}

#[derive(Clone, PartialEq, Eq)]
enum OwnedObjectStorageKey {
    Nar(NarFileName),
    NarInfo(CacheId, NarInfoFileName),
}

/// The type of error returned by [`InMemoryObjectStorage`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum InMemoryObjectStorageError {
    /// The multipart upload was completed before being started.
    #[display("multipart upload was not started")]
    MultipartUploadNotStarted,

    /// A multipart operation used a key different from the one used to create
    /// the upload.
    #[display("multipart upload key does not match its initiation key")]
    MultipartUploadKeyMismatch,

    /// A multipart upload part was missing when completing the upload.
    #[display("multipart upload is missing bytes for part {_0}")]
    MissingMultipartUploadPart(usize),

    /// Multipart upload completion had fewer Etags than uploaded part slots.
    #[display(
        "multipart upload completion has too few Etags: {_0} for {_1} parts"
    )]
    TooFewMultipartUploadEtags(usize, usize),

    /// Multipart upload completion had more Etags than uploaded part slots.
    #[display(
        "multipart upload completion has too many Etags: {_0} for {_1} parts"
    )]
    TooManyMultipartUploadEtags(usize, usize),
}

impl InMemoryObjectStorage {
    fn with_inner<R>(
        &self,
        f: impl FnOnce(&mut InMemoryObjectStorageInner) -> R,
    ) -> R {
        #[expect(clippy::unwrap_used)]
        let mut inner = self.0.lock().unwrap();
        f(&mut inner)
    }

    /// Stores a part in an in-progress multipart upload.
    pub fn upload_multipart_part(
        &self,
        upload_id: MultipartUploadId,
        part_number: NonZeroU16,
        bytes: Bytes,
    ) -> Result<Etag, InMemoryObjectStorageError> {
        self.with_inner(|inner| {
            let upload = inner
                .multipart_uploads
                .get_mut(&upload_id)
                .ok_or(InMemoryObjectStorageError::MultipartUploadNotStarted)?;

            let part_idx = usize::from(part_number.get() - 1);
            if upload.parts.len() <= part_idx {
                upload.parts.resize(part_idx + 1, None);
            }
            upload.parts[part_idx] = Some(bytes);

            Ok("d41d8cd98f00b204e9800998ecf8427e"
                .parse()
                .expect("valid test Etag"))
        })
    }
}

impl InMemoryObjectStorageInner {
    fn get(&self, key: ObjectStorageKey<'_>) -> Option<&Bytes> {
        match key {
            ObjectStorageKey::Nar(key) => self.nars.get(key),
            ObjectStorageKey::NarInfo(cache_id, narinfo_file_name) => {
                self.nar_infos.get(&(*cache_id, narinfo_file_name.clone()))
            },
        }
    }

    fn insert(&mut self, key: ObjectStorageKey<'_>, bytes: Bytes) {
        match key {
            ObjectStorageKey::Nar(nar_file_name) => {
                self.nars.insert(nar_file_name.clone(), bytes);
            },
            ObjectStorageKey::NarInfo(cache_id, narinfo_file_name) => {
                self.nar_infos
                    .insert((*cache_id, narinfo_file_name.clone()), bytes);
            },
        }
    }
}

impl From<ObjectStorageKey<'_>> for OwnedObjectStorageKey {
    fn from(key: ObjectStorageKey<'_>) -> Self {
        match key {
            ObjectStorageKey::Nar(nar_file_name) => {
                Self::Nar(nar_file_name.clone())
            },
            ObjectStorageKey::NarInfo(cache_id, narinfo_file_name) => {
                Self::NarInfo(*cache_id, narinfo_file_name.clone())
            },
        }
    }
}

impl ObjectStorage for InMemoryObjectStorage {
    type MultipartUploadId = MultipartUploadId;
    type Error = InMemoryObjectStorageError;

    async fn contains_key(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_inner(|inner| inner.get(key).is_some()))
    }

    async fn get(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.with_inner(|inner| inner.get(key).cloned()))
    }

    async fn insert(
        &self,
        key: ObjectStorageKey<'_>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<bool, Self::Error> {
        self.with_inner(|inner| {
            if inner.get(key).is_some() {
                return Ok(false);
            }

            inner.insert(key, Bytes::copy_from_slice(bytes.as_ref()));
            Ok(true)
        })
    }

    async fn create_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
    ) -> Result<Self::MultipartUploadId, Self::Error> {
        Ok(self.with_inner(|inner| {
            let upload_id = inner.next_upload_id;
            inner.next_upload_id += 1;
            inner.multipart_uploads.insert(
                upload_id,
                MultipartUpload { key: key.into(), parts: Vec::new() },
            );
            upload_id
        }))
    }

    async fn abort_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
        upload_id: Self::MultipartUploadId,
    ) -> Result<(), Self::Error> {
        self.with_inner(|inner| {
            let upload = inner
                .multipart_uploads
                .get(&upload_id)
                .ok_or(InMemoryObjectStorageError::MultipartUploadNotStarted)?;

            if upload.key != key.into() {
                return Err(
                    InMemoryObjectStorageError::MultipartUploadKeyMismatch,
                );
            }

            inner.multipart_uploads.remove(&upload_id);
            Ok(())
        })
    }

    async fn complete_multipart_upload(
        &self,
        key: ObjectStorageKey<'_>,
        upload_id: Self::MultipartUploadId,
        etags: &[Etag],
    ) -> Result<bool, Self::Error> {
        self.with_inner(|inner| {
            if inner.get(key).is_some() {
                return Ok(false);
            }

            let upload = inner
                .multipart_uploads
                .get_mut(&upload_id)
                .ok_or(InMemoryObjectStorageError::MultipartUploadNotStarted)?;

            if upload.key != key.into() {
                return Err(
                    InMemoryObjectStorageError::MultipartUploadKeyMismatch,
                );
            }

            if etags.len() < upload.parts.len() {
                return Err(
                    InMemoryObjectStorageError::TooFewMultipartUploadEtags(
                        etags.len(),
                        upload.parts.len(),
                    ),
                );
            }

            if etags.len() > upload.parts.len() {
                return Err(
                    InMemoryObjectStorageError::TooManyMultipartUploadEtags(
                        etags.len(),
                        upload.parts.len(),
                    ),
                );
            }

            let mut bytes = Vec::new();
            for (part_idx, maybe_part) in upload.parts.iter_mut().enumerate() {
                let part = maybe_part.take().ok_or(
                    InMemoryObjectStorageError::MissingMultipartUploadPart(
                        part_idx + 1,
                    ),
                )?;
                bytes.extend_from_slice(&part);
            }

            inner.multipart_uploads.remove(&upload_id);
            inner.insert(key, Bytes::from(bytes));
            Ok(true)
        })
    }

    fn max_multipart_parts(&self) -> Option<usize> {
        None
    }

    fn part_size(&self) -> NonZeroU32 {
        NonZeroU32::MAX
    }
}
