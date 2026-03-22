use core::fmt;
use std::sync::LazyLock;

use axum::http;
use cache::object_gateway::StorageAccessRequest;
use cache::object_storage::ObjectStorageKey;
use cache::{ObjectGateway, ObjectStorage};
use worker::send::IntoSendFuture;

/// The host of the S3-compatible endpoint for the R2 cache bucket.
static S3_ENDPOINT_HOST: LazyLock<String> = LazyLock::new(|| {
    format!("{}.r2.cloudflarestorage.com", crate::CLOUDFLARE_ACCOUNT_ID)
});

/// The [`ObjectStorage`] and [`ObjectGateway`] implementation backed by
/// Cloudflare R2.
#[derive(Debug, Clone)]
pub(crate) struct R2ObjectStorage {
    bucket: worker::Bucket,
    s3_creds: R2S3Credentials,
}

/// S3-compatible credentials for generating presigned R2 URLs.
///
/// These are obtained by creating an R2 API token in the Cloudflare dashboard.
/// The token must have "Object Read & Write" permission scoped to the target
/// bucket.
#[derive(Debug, Clone)]
struct R2S3Credentials {
    /// The R2 API token's Access Key ID.
    access_key_id: String,
    /// The R2 API token's Secret Access Key.
    secret_access_key: String,
}

struct R2BucketKey<'a>(ObjectStorageKey<'a>);

impl R2ObjectStorage {
    pub(crate) fn new(env: &worker::Env) -> Self {
        Self {
            bucket: env
                .bucket(env!("R2_BUCKET_BINDING_NAME"))
                .expect("R2 bucket binding not found"),
            s3_creds: R2S3Credentials::new(env),
        }
    }
}

impl R2S3Credentials {
    fn new(env: &worker::Env) -> Self {
        Self {
            access_key_id: env
                .secret("R2_ACCESS_KEY_ID")
                .expect("R2_ACCESS_KEY_ID secret not found")
                .to_string(),
            secret_access_key: env
                .secret("R2_SECRET_ACCESS_KEY")
                .expect("R2_SECRET_ACCESS_KEY secret not found")
                .to_string(),
        }
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

    fn object_url(
        &self,
        access_request: StorageAccessRequest<'_>,
    ) -> http::Uri {
        // R2 doesn't support unauthenticated writes, so we authenticate
        // via a presigned URL. R2 only supports presigned URLs on the
        // S3-compatible endpoint, not on custom domains (see
        // https://developers.cloudflare.com/r2/api/s3/presigned-urls/#custom-domains).
        //
        // TODO: switch to authenticating via WAF HMAC validation once it makes
        // sense to start paying for a Cloudflare Pro plan.
        if access_request.method == http::Method::PUT {
            let base_url = format!(
                "https://{}/{}/{}",
                &*S3_ENDPOINT_HOST,
                env!("R2_BUCKET_NAME"),
                R2BucketKey(access_request.key),
            )
            .parse::<url::Url>()
            .expect("valid S3 URI");

            let now = chrono::DateTime::from_timestamp_millis(
                js_sys::Date::now() as i64,
            )
            .expect("current time is a valid timestamp");

            s3v4::pre_signed_url(
                &self.s3_creds.access_key_id,
                &self.s3_creds.secret_access_key,
                access_request.min_ttl.as_secs(),
                &base_url,
                access_request.method.as_str(),
                "UNSIGNED-PAYLOAD",
                "auto",
                &now,
                "s3",
            )
            .expect("presigning should not fail")
            .parse()
            .expect("presigned URL is a valid URI")
        } else {
            format!("/{}", R2BucketKey(access_request.key))
                .parse()
                .expect("valid relative URI")
        }
    }

    fn storage(&self) -> &Self {
        self
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
