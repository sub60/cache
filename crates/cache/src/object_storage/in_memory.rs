use core::num::NonZeroU32;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use nix_types::sub60::nar_multiparts::Etag;
use nix_types::{NarFileName, NarInfoFileName, StoreBasename};

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;

/// An [`ObjectStorage`] trait that stores the object in memory using a
/// `HashMap`.
#[derive(Clone, Default)]
pub struct InMemoryObjectStorage(Arc<Mutex<InMemoryObjectStorageInner>>);

#[derive(Default)]
struct InMemoryObjectStorageInner {
    nars: HashMap<NarFileName, ObjectState>,
    nar_infos: HashMap<NarInfoFileName, ObjectState>,
    nar_multiparts: HashMap<StoreBasename, ObjectState>,
}

enum ObjectState {
    Complete(Bytes),
    Incomplete(Vec<Option<Bytes>>),
}

/// The type of error returned by [`InMemoryObjectStorage`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum InMemoryObjectStorageError {
    /// The source object for a move operation does not exist.
    #[display("move source object was not found")]
    MoveSourceNotFound,

    /// The multipart upload was completed before being started.
    #[display("multipart upload was not started")]
    MultipartUploadNotStarted,

    /// The multipart upload has already been completed.
    #[display("multipart upload is already complete")]
    MultipartUploadAlreadyComplete,

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
}

impl InMemoryObjectStorageInner {
    fn get(&mut self, key: ObjectStorageKey<'_>) -> Option<&ObjectState> {
        match key {
            ObjectStorageKey::Nar(key) => self.nars.get(key),
            ObjectStorageKey::NarInfo(key) => self.nar_infos.get(key),
            ObjectStorageKey::NarMultipart(key) => self.nar_multiparts.get(key),
        }
    }

    fn get_mut(
        &mut self,
        key: ObjectStorageKey<'_>,
    ) -> Option<&mut ObjectState> {
        match key {
            ObjectStorageKey::Nar(key) => self.nars.get_mut(key),
            ObjectStorageKey::NarInfo(key) => self.nar_infos.get_mut(key),
            ObjectStorageKey::NarMultipart(key) => {
                self.nar_multiparts.get_mut(key)
            },
        }
    }

    fn insert(&mut self, key: ObjectStorageKey<'_>, obj: ObjectState) {
        match key {
            ObjectStorageKey::Nar(nar_file_name) => {
                self.nars.insert(nar_file_name.clone(), obj);
            },
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                self.nar_infos.insert(narinfo_file_name.clone(), obj);
            },
            ObjectStorageKey::NarMultipart(store_basename) => {
                self.nar_multiparts.insert(store_basename.clone(), obj);
            },
        }
    }

    fn remove(&mut self, key: ObjectStorageKey<'_>) -> Option<ObjectState> {
        match key {
            ObjectStorageKey::Nar(nar_file_name) => {
                self.nars.remove(nar_file_name)
            },
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                self.nar_infos.remove(narinfo_file_name)
            },
            ObjectStorageKey::NarMultipart(store_basename) => {
                self.nar_multiparts.remove(store_basename)
            },
        }
    }
}

impl ObjectStorage for InMemoryObjectStorage {
    type MultipartUploadId = String;
    type Error = InMemoryObjectStorageError;

    async fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_inner(|inner| {
            inner
                .get(key.into())
                .is_some_and(|state| matches!(state, ObjectState::Complete(_)))
        }))
    }

    async fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.with_inner(|inner| {
            inner.get(key.into()).and_then(|state| match state {
                ObjectState::Complete(bytes) => Some(bytes.clone()),
                ObjectState::Incomplete(_) => None,
            })
        }))
    }

    async fn set<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), Self::Error> {
        self.with_inner(|inner| {
            inner.insert(
                key.into(),
                ObjectState::Complete(bytes.as_ref().to_owned().into()),
            )
        });

        Ok(())
    }

    async fn r#move<'src, 'dst>(
        &self,
        source: impl Into<ObjectStorageKey<'src>> + Send,
        destination: impl Into<ObjectStorageKey<'dst>> + Send,
    ) -> Result<(), Self::Error> {
        self.with_inner(|inner| {
            let object = inner
                .remove(source.into())
                .ok_or(InMemoryObjectStorageError::MoveSourceNotFound)?;

            inner.insert(destination.into(), object);

            Ok(())
        })
    }

    async fn create_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
    ) -> Result<Self::MultipartUploadId, Self::Error> {
        self.with_inner(|inner| {
            inner.insert(key.into(), ObjectState::Incomplete(Vec::new()))
        });

        Ok("in-memory".to_owned())
    }

    async fn complete_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>> + Send,
        _: Self::MultipartUploadId,
        etags: &[Etag],
    ) -> Result<(), Self::Error> {
        self.with_inner(|inner| {
            let state = inner
                .get_mut(key.into())
                .ok_or(InMemoryObjectStorageError::MultipartUploadNotStarted)?;

            let ObjectState::Incomplete(parts) = state else {
                return Err(
                    InMemoryObjectStorageError::MultipartUploadAlreadyComplete,
                );
            };

            if etags.len() < parts.len() {
                return Err(
                    InMemoryObjectStorageError::TooFewMultipartUploadEtags(
                        etags.len(),
                        parts.len(),
                    ),
                );
            }

            if etags.len() > parts.len() {
                return Err(
                    InMemoryObjectStorageError::TooManyMultipartUploadEtags(
                        etags.len(),
                        parts.len(),
                    ),
                );
            }

            let mut bytes = Vec::new();

            for (part_idx, maybe_part) in parts.iter_mut().enumerate() {
                let part = maybe_part.take().ok_or(
                    InMemoryObjectStorageError::MissingMultipartUploadPart(
                        part_idx + 1,
                    ),
                )?;
                bytes.extend_from_slice(&part);
            }

            *state = ObjectState::Complete(Bytes::copy_from_slice(&bytes));

            Ok(())
        })
    }

    fn max_multipart_parts(&self) -> Option<usize> {
        None
    }

    fn part_size(&self) -> NonZeroU32 {
        NonZeroU32::MAX
    }
}
