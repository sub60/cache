use core::fmt;

use axum::http;
use cache::object_storage::ObjectStorageKey;
use cache::{ObjectGateway, ObjectStorage};
use types::NarFileName;
use worker::send::IntoSendFuture;

/// The [`ObjectStorage`] and [`ObjectGateway`] implementation backed by
/// Cloudflare R2.
#[derive(Debug, Clone)]
pub struct R2ObjectStorage {
    bucket: worker::Bucket,
}

struct R2BucketKey<'a>(ObjectStorageKey<'a>);

impl R2ObjectStorage {
    /// Creates a new [`R2ObjectStorage`] backed by the given R2 bucket.
    pub fn new(bucket: worker::Bucket) -> Self {
        Self { bucket }
    }
}

impl ObjectStorage for R2ObjectStorage {
    type Error = worker::Error;

    async fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        self.bucket
            .head(R2BucketKey(key.into()))
            .into_send()
            .await
            .map(|obj| obj.is_some())
    }

    async fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<impl AsRef<[u8]>>, Self::Error> {
        let Some(object) = self
            .bucket
            .get(R2BucketKey(key.into()))
            .execute()
            .into_send()
            .await?
        else {
            return Ok(None);
        };

        object
            .body()
            .expect("GET response should always have a body")
            .bytes()
            .into_send()
            .await
            .map(Some)
    }

    async fn set<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), Self::Error> {
        self.bucket
            .put(R2BucketKey(key.into()), bytes.as_ref().to_vec())
            .execute()
            .into_send()
            .await
            .map(|_obj| ())
    }
}

impl ObjectGateway for R2ObjectStorage {
    type Storage = Self;

    fn nar_url(&self, filename: &NarFileName) -> http::Uri {
        format!("/{}", R2BucketKey(ObjectStorageKey::Nar(filename)))
            .parse()
            .expect("valid URI")
    }

    fn storage(&self) -> &Self {
        self
    }
}

impl<'a> From<ObjectStorageKey<'a>> for R2BucketKey<'a> {
    fn from(key: ObjectStorageKey<'a>) -> Self {
        Self(key)
    }
}

impl fmt::Display for R2BucketKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            // This `nar/` prefix is coupled to a Cloudflare Cloud Connector
            // rule that routes requests whose URI path starts with `/nar/` on
            // `cache.sub60.dev` directly to the R2 bucket, bypassing the
            // worker. If this prefix changes, the Cloud Connector rule must be
            // updated to match.
            ObjectStorageKey::Nar(nar) => write!(f, "nar/{nar}"),

            // Unlike NARs, NARInfos are always read and written directly
            // through the cache worker's R2 bucket binding, so this `narinfo/`
            // prefix is purely an internal key namespace and is never exposed
            // publicly.
            ObjectStorageKey::NarInfo(narinfo) => {
                write!(f, "narinfo/{narinfo}")
            },
        }
    }
}

impl From<R2BucketKey<'_>> for String {
    fn from(key: R2BucketKey<'_>) -> Self {
        key.to_string()
    }
}
