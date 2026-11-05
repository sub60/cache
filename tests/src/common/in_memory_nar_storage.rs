use core::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use core::time::Duration;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{self, Bytes};
use axum::extract::{Path, Request};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use nix_types::NarFileName;
use sha2::{Digest, Sha256};
use sub60_cache::api::Etag;
use sub60_cache::{
    CacheInfos,
    CompleteMultipartUploadError,
    MultipartUpload,
    NarStorage,
};

/// The ID of an in-memory multipart upload.
pub type MultipartUploadId = u64;

/// A [`NarStorage`] that stores NARs and multipart uploads in memory.
///
/// It serves the NARs it stores at the URLs it hands out once it's converted
/// into a [`Router`] and merged into the app's router.
#[derive(Clone)]
pub struct InMemoryNarStorage {
    base_route: Cow<'static, str>,
    inner: Arc<Mutex<InMemoryNarStorageInner>>,
}

struct InMemoryNarStorageInner {
    nars: HashMap<NarFileName, Bytes>,
    multipart_uploads: HashMap<MultipartUploadId, PendingUpload>,
    next_upload_id: MultipartUploadId,
    part_size: NonZeroU32,
}

struct PendingUpload {
    nar: NarFileName,
    /// The size of the NAR the upload declared when it started, in bytes.
    declared_size: NonZeroU64,
    parts: Vec<Option<Bytes>>,
}

/// The type of error returned by [`InMemoryNarStorage`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum InMemoryNarStorageError {
    /// The SHA-256 hash of an inserted NAR isn't the one in its file name.
    #[display("NAR's hash doesn't match its file name")]
    HashMismatch,

    /// The multipart upload was completed before being started.
    #[display("multipart upload was not started")]
    MultipartUploadNotStarted,

    /// A part of a multipart upload was uploaded with a different length than
    /// its URL accepts, which is the part size for every part but the last.
    #[display("part {part_number} has {len} bytes instead of {expected_len}")]
    PartLengthMismatch {
        part_number: NonZeroU16,
        len: usize,
        expected_len: u64,
    },

    /// A part past the end of the upload's NAR was requested or uploaded.
    #[display("part {_0} is past the end of the NAR")]
    PartPastEnd(NonZeroU16),

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

    /// The body of an inserted NAR couldn't be read.
    #[display("couldn't read NAR from request body")]
    UnreadableBody,
}

impl InMemoryNarStorage {
    /// Inserts a NAR, unless one with the same file name is already stored.
    ///
    /// Unlike [`NarStorage::insert()`], it doesn't check that the bytes'
    /// SHA-256 hash is the one in the NAR's file name, so tests can seed the
    /// storage with any bytes.
    ///
    /// Returns `true` if the NAR was inserted.
    pub fn insert_unchecked(
        &self,
        nar: &NarFileName,
        bytes: impl Into<Bytes>,
    ) -> bool {
        self.with_inner(|inner| {
            if inner.nars.contains_key(nar) {
                return false;
            }
            inner.nars.insert(nar.clone(), bytes.into());
            true
        })
    }

    /// Creates a new [`InMemoryNarStorage`] whose NARs are served at the given
    /// route path.
    ///
    /// # Panics
    ///
    /// Panics if the base route path does not start with a slash or if it
    /// contains path captures (e.g., `/{foo}`).
    pub fn new(base_route: &str) -> Self {
        assert!(
            base_route.starts_with('/'),
            "route path must start with a slash"
        );
        assert!(
            !base_route.contains('{') && !base_route.contains('}'),
            "base route must not contain path captures (`{{...}}`)"
        );
        let base_route = if base_route == "/" {
            Cow::Borrowed("")
        } else {
            Cow::Owned(base_route.trim_end_matches('/').to_owned())
        };
        Self { base_route, inner: Default::default() }
    }

    /// Sets the size of the parts of multipart uploads, which is larger than
    /// any NAR by default.
    pub fn set_part_size(&self, part_size: NonZeroU32) {
        self.with_inner(|inner| inner.part_size = part_size);
    }

    /// Stores a part in an in-progress multipart upload.
    ///
    /// Like the URLs of production storages, it only accepts a part of the
    /// length the upload's declared size leaves for it.
    pub fn upload_multipart_part(
        &self,
        upload_id: MultipartUploadId,
        part_number: NonZeroU16,
        bytes: Bytes,
    ) -> Result<Etag, InMemoryNarStorageError> {
        self.with_inner(|inner| {
            let part_size = inner.part_size;

            let upload = inner
                .multipart_uploads
                .get_mut(&upload_id)
                .ok_or(InMemoryNarStorageError::MultipartUploadNotStarted)?;

            let expected_len =
                part_len(upload.declared_size, part_size, part_number)?;

            if bytes.len() as u64 != expected_len {
                return Err(InMemoryNarStorageError::PartLengthMismatch {
                    part_number,
                    len: bytes.len(),
                    expected_len,
                });
            }

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

    fn nar_url(&self, nar: &NarFileName) -> String {
        format!("{}/{nar}", self.base_route)
    }

    fn with_inner<R>(
        &self,
        f: impl FnOnce(&mut InMemoryNarStorageInner) -> R,
    ) -> R {
        #[expect(clippy::unwrap_used)]
        let mut inner = self.inner.lock().unwrap();
        f(&mut inner)
    }
}

/// Returns the length of the given part of a multipart upload of a NAR of the
/// given size, in bytes.
///
/// Every part is a full part except the last one, which gets what's left, so
/// production storages only accept those lengths.
fn part_len(
    declared_size: NonZeroU64,
    part_size: NonZeroU32,
    part_number: NonZeroU16,
) -> Result<u64, InMemoryNarStorageError> {
    let part_size = u64::from(part_size.get());
    let start = u64::from(part_number.get() - 1) * part_size;

    declared_size
        .get()
        .checked_sub(start)
        .filter(|&rest| rest > 0)
        .map(|rest| rest.min(part_size))
        .ok_or(InMemoryNarStorageError::PartPastEnd(part_number))
}

impl From<InMemoryNarStorage> for Router {
    fn from(storage: InMemoryNarStorage) -> Self {
        let get_storage = storage.clone();

        let get_handler = async move |Path::<String>(nar)| {
            let nar = match nar.parse::<NarFileName>() {
                Ok(nar) => nar,
                Err(err) => {
                    let msg = format!("invalid NarFileName: {err}");
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            match get_storage.with_inner(|inner| inner.nars.get(&nar).cloned())
            {
                Some(bytes) => Response::new(bytes.into()),
                None => StatusCode::NOT_FOUND.into_response(),
            }
        };

        let multipart_put_handler = async || StatusCode::NOT_IMPLEMENTED;

        let route_path = format!("{}/{{nar}}", storage.base_route);

        let multipart_route_path =
            format!("{}/multipart-upload/{{nar}}", storage.base_route);

        Self::new()
            .route(&multipart_route_path, put(multipart_put_handler))
            .route(&route_path, get(get_handler))
    }
}

impl Default for InMemoryNarStorageInner {
    fn default() -> Self {
        Self {
            nars: HashMap::new(),
            multipart_uploads: HashMap::new(),
            next_upload_id: 0,
            part_size: NonZeroU32::MAX,
        }
    }
}

impl NarStorage for InMemoryNarStorage {
    type MultipartUploadId = MultipartUploadId;
    type Error = InMemoryNarStorageError;

    async fn complete_multipart_upload(
        &self,
        upload: MultipartUpload<'_, Self::MultipartUploadId>,
        etags: &[Etag],
    ) -> Result<bool, CompleteMultipartUploadError<Self::Error>> {
        self.with_inner(|inner| {
            // The NAR is already stored, so the upload's parts are discarded.
            if inner.nars.contains_key(upload.nar) {
                if inner
                    .multipart_uploads
                    .get(upload.id)
                    .is_some_and(|pending| &pending.nar == upload.nar)
                {
                    inner.multipart_uploads.remove(upload.id);
                }
                return Ok(false);
            }

            // Upload IDs are scoped to the NAR they were created for, so an
            // upload created for a different NAR doesn't exist for this one.
            let pending = inner
                .multipart_uploads
                .get_mut(upload.id)
                .filter(|pending| &pending.nar == upload.nar)
                .ok_or(CompleteMultipartUploadError::NoSuchUpload)?;

            if etags.len() < pending.parts.len() {
                return Err(CompleteMultipartUploadError::InvalidParts(
                    InMemoryNarStorageError::TooFewMultipartUploadEtags(
                        etags.len(),
                        pending.parts.len(),
                    ),
                ));
            }

            if etags.len() > pending.parts.len() {
                return Err(CompleteMultipartUploadError::InvalidParts(
                    InMemoryNarStorageError::TooManyMultipartUploadEtags(
                        etags.len(),
                        pending.parts.len(),
                    ),
                ));
            }

            let mut bytes = Vec::new();
            for (part_idx, maybe_part) in pending.parts.iter_mut().enumerate() {
                let part = maybe_part.take().ok_or(
                    CompleteMultipartUploadError::InvalidParts(
                        InMemoryNarStorageError::MissingMultipartUploadPart(
                            part_idx + 1,
                        ),
                    ),
                )?;
                bytes.extend_from_slice(&part);
            }

            inner.multipart_uploads.remove(upload.id);
            inner.nars.insert(upload.nar.clone(), Bytes::from(bytes));
            Ok(true)
        })
    }

    async fn contains(
        &self,
        _cache: &CacheInfos,
        nar: &NarFileName,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_inner(|inner| inner.nars.contains_key(nar)))
    }

    async fn create_multipart_upload(
        &self,
        _cache: &CacheInfos,
        nar: &NarFileName,
        size: NonZeroU64,
    ) -> Result<Self::MultipartUploadId, Self::Error> {
        Ok(self.with_inner(|inner| {
            let upload_id = inner.next_upload_id;
            inner.next_upload_id += 1;
            inner.multipart_uploads.insert(
                upload_id,
                PendingUpload {
                    nar: nar.clone(),
                    declared_size: size,
                    parts: Vec::new(),
                },
            );
            upload_id
        }))
    }

    fn download_url(
        &self,
        _cache: &CacheInfos,
        nar: &NarFileName,
    ) -> impl AsRef<str> {
        self.nar_url(nar)
    }

    async fn get(
        &self,
        _cache: &CacheInfos,
        nar: &NarFileName,
    ) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.with_inner(|inner| inner.nars.get(nar).cloned()))
    }

    async fn insert(
        &self,
        _cache: &CacheInfos,
        nar: &NarFileName,
        request: Request,
    ) -> Result<bool, Self::Error> {
        let bytes = body::to_bytes(request.into_body(), usize::MAX)
            .await
            .map_err(|_| InMemoryNarStorageError::UnreadableBody)?;

        if Sha256::digest(&bytes).as_slice() != nar.file_hash.to_bytes() {
            return Err(InMemoryNarStorageError::HashMismatch);
        }

        Ok(self.insert_unchecked(nar, bytes))
    }

    fn max_multipart_parts(&self) -> Option<usize> {
        None
    }

    async fn multipart_upload_part_urls(
        &self,
        upload: MultipartUpload<'_, Self::MultipartUploadId>,
        part_numbers: &[NonZeroU16],
        _min_ttl: Duration,
    ) -> Result<Vec<String>, Self::Error> {
        let (declared_size, part_size) = self.with_inner(|inner| {
            inner
                .multipart_uploads
                .get(upload.id)
                .map(|pending| (pending.declared_size, inner.part_size))
                .ok_or(InMemoryNarStorageError::MultipartUploadNotStarted)
        })?;

        part_numbers
            .iter()
            .map(|&part_number| {
                part_len(declared_size, part_size, part_number)?;
                Ok(format!(
                    "http://cache.test{}/multipart-upload/{}?partNumber={part_number}&uploadId={}",
                    self.base_route, upload.nar, upload.id,
                ))
            })
            .collect()
    }

    fn part_size(&self) -> NonZeroU32 {
        self.with_inner(|inner| inner.part_size)
    }
}

impl From<InMemoryNarStorageError> for StatusCode {
    fn from(err: InMemoryNarStorageError) -> Self {
        match err {
            InMemoryNarStorageError::HashMismatch
            | InMemoryNarStorageError::PartLengthMismatch { .. }
            | InMemoryNarStorageError::PartPastEnd(_) => Self::BAD_REQUEST,
            InMemoryNarStorageError::MultipartUploadNotStarted => {
                Self::NOT_FOUND
            },
            InMemoryNarStorageError::MissingMultipartUploadPart(_)
            | InMemoryNarStorageError::TooFewMultipartUploadEtags(..)
            | InMemoryNarStorageError::TooManyMultipartUploadEtags(..)
            | InMemoryNarStorageError::UnreadableBody => {
                Self::INTERNAL_SERVER_ERROR
            },
        }
    }
}
