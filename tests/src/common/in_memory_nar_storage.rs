use core::num::{NonZeroU16, NonZeroU32};
use core::time::Duration;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use nix_types::NarFileName;
use sub60_cache::api::Etag;
use sub60_cache::{
    CacheId,
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

#[derive(Default)]
struct InMemoryNarStorageInner {
    nars: HashMap<NarFileName, Bytes>,
    multipart_uploads: HashMap<MultipartUploadId, PendingUpload>,
    next_upload_id: MultipartUploadId,
}

struct PendingUpload {
    nar: NarFileName,
    parts: Vec<Option<Bytes>>,
}

/// The type of error returned by [`InMemoryNarStorage`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum InMemoryNarStorageError {
    /// The multipart upload was completed before being started.
    #[display("multipart upload was not started")]
    MultipartUploadNotStarted,

    /// A multipart operation used a NAR different from the one used to create
    /// the upload.
    #[display("multipart upload NAR does not match its initiation NAR")]
    MultipartUploadNarMismatch,

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

impl InMemoryNarStorage {
    /// Inserts a NAR, unless one with the same file name is already stored.
    ///
    /// Returns `true` if the NAR was inserted.
    pub fn insert(&self, nar: &NarFileName, bytes: impl Into<Bytes>) -> bool {
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

    /// Stores a part in an in-progress multipart upload.
    pub fn upload_multipart_part(
        &self,
        upload_id: MultipartUploadId,
        part_number: NonZeroU16,
        bytes: Bytes,
    ) -> Result<Etag, InMemoryNarStorageError> {
        self.with_inner(|inner| {
            let upload = inner
                .multipart_uploads
                .get_mut(&upload_id)
                .ok_or(InMemoryNarStorageError::MultipartUploadNotStarted)?;

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

        let put_storage = storage.clone();

        let put_handler = async move |Path::<String>(nar), bytes: Bytes| {
            let nar = match nar.parse::<NarFileName>() {
                Ok(nar) => nar,
                Err(err) => {
                    let msg = format!("invalid NarFileName: {err}");
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            if put_storage.insert(&nar, bytes) {
                StatusCode::CREATED.into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        };

        let multipart_put_handler = async || StatusCode::NOT_IMPLEMENTED;

        let route_path = format!("{}/{{nar}}", storage.base_route);

        let multipart_route_path =
            format!("{}/multipart-upload/{{nar}}", storage.base_route);

        Self::new()
            .route(&multipart_route_path, put(multipart_put_handler))
            .route(&route_path, get(get_handler).put(put_handler))
    }
}

impl NarStorage for InMemoryNarStorage {
    type MultipartUploadId = MultipartUploadId;
    type Error = InMemoryNarStorageError;

    async fn abort_multipart_upload(
        &self,
        upload: MultipartUpload<'_, Self::MultipartUploadId>,
    ) -> Result<bool, Self::Error> {
        self.with_inner(|inner| {
            let Some(pending) = inner.multipart_uploads.get(upload.id) else {
                return Ok(false);
            };

            if &pending.nar != upload.nar {
                return Err(
                    InMemoryNarStorageError::MultipartUploadNarMismatch,
                );
            }

            inner.multipart_uploads.remove(upload.id);
            Ok(true)
        })
    }

    async fn complete_multipart_upload(
        &self,
        upload: MultipartUpload<'_, Self::MultipartUploadId>,
        etags: &[Etag],
    ) -> Result<bool, CompleteMultipartUploadError<Self::Error>> {
        self.with_inner(|inner| {
            if inner.nars.contains_key(upload.nar) {
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
        _cache_id: &CacheId,
        nar: &NarFileName,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_inner(|inner| inner.nars.contains_key(nar)))
    }

    async fn create_multipart_upload(
        &self,
        _cache_id: &CacheId,
        nar: &NarFileName,
    ) -> Result<Self::MultipartUploadId, Self::Error> {
        Ok(self.with_inner(|inner| {
            let upload_id = inner.next_upload_id;
            inner.next_upload_id += 1;
            inner.multipart_uploads.insert(
                upload_id,
                PendingUpload { nar: nar.clone(), parts: Vec::new() },
            );
            upload_id
        }))
    }

    fn download_url(
        &self,
        _cache_id: &CacheId,
        nar: &NarFileName,
    ) -> impl AsRef<str> {
        self.nar_url(nar)
    }

    fn max_multipart_parts(&self) -> Option<usize> {
        None
    }

    fn multipart_upload_part_url(
        &self,
        upload: MultipartUpload<'_, Self::MultipartUploadId>,
        part_number: NonZeroU16,
        _min_ttl: Duration,
    ) -> impl AsRef<str> {
        format!(
            "http://cache.test{}/multipart-upload/{}?partNumber={part_number}&uploadId={}",
            self.base_route, upload.nar, upload.id,
        )
    }

    fn part_size(&self) -> NonZeroU32 {
        NonZeroU32::MAX
    }

    fn upload_url(
        &self,
        _cache_id: &CacheId,
        nar: &NarFileName,
        _min_ttl: Duration,
    ) -> impl AsRef<str> {
        self.nar_url(nar)
    }
}
