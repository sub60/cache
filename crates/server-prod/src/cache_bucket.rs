use core::fmt;
use std::sync::{LazyLock, OnceLock};

use axum::http;
use cache::object_gateway::StorageAccessRequest;
use cache::object_storage::ObjectStorageKey;
use cache::{ObjectGateway, ObjectStorage};
use worker::send::IntoSendFuture;

static PUBLIC_ENDPOINT: LazyLock<url::Url> = LazyLock::new(|| {
    url::Url::parse("https://r2.cache.sub60.dev").expect("valid URL")
});

static S3_ENDPOINT: LazyLock<url::Url> = LazyLock::new(|| {
    // NOTE: the trailing "/" matters! Without it the bucket name would be
    // replaced by the bucket key when we construct the base URL for
    // pre-signing.
    format!(
        "https://{}.r2.cloudflarestorage.com/{}/",
        crate::CLOUDFLARE_ACCOUNT_ID,
        env!("R2_BUCKET_NAME")
    )
    .parse::<url::Url>()
    .expect("valid URL")
});

/// The [`ObjectStorage`] and [`ObjectGateway`] implementation backed by
/// Cloudflare R2.
#[derive(Debug, Clone)]
pub(crate) struct CacheBucket {
    env: worker::Env,
    inner: worker::Bucket,
    s3_creds: OnceLock<CacheBucketS3Credentials>,
}

/// S3-compatible credentials for generating presigned R2 URLs.
///
/// These are obtained by creating an R2 API token in the Cloudflare dashboard
/// with "Object Read & Write" permissions scoped to the cache bucket.
#[derive(Debug, Clone)]
struct CacheBucketS3Credentials {
    /// The R2 API token's Access Key ID.
    access_key_id: String,
    /// The R2 API token's Secret Access Key.
    secret_access_key: String,
}

struct CacheBucketKey<'a>(ObjectStorageKey<'a>);

impl CacheBucket {
    pub(crate) fn new(env: worker::Env) -> Self {
        Self {
            inner: env
                .bucket(env!("R2_BUCKET_BINDING_NAME"))
                .expect("R2 bucket binding not found"),
            env,
            s3_creds: OnceLock::new(),
        }
    }

    fn s3_creds(&self) -> &CacheBucketS3Credentials {
        self.s3_creds.get_or_init(|| CacheBucketS3Credentials::new(&self.env))
    }
}

impl CacheBucketS3Credentials {
    fn new(env: &worker::Env) -> Self {
        Self {
            access_key_id: env
                .var("CACHE_BUCKET_ACCESS_KEY_ID")
                .expect("CACHE_BUCKET_ACCESS_KEY_ID variable not found")
                .to_string(),
            secret_access_key: env
                .secret("CACHE_BUCKET_SECRET_ACCESS_KEY")
                .expect("CACHE_BUCKET_SECRET_ACCESS_KEY secret not found")
                .to_string(),
        }
    }
}

impl ObjectStorage for CacheBucket {
    type Error = worker::Error;

    async fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        self.inner
            .head(CacheBucketKey(key.into()))
            .into_send()
            .await
            .map(|obj| obj.is_some())
    }

    async fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<impl AsRef<[u8]>>, Self::Error> {
        let Some(object) = self
            .inner
            .get(CacheBucketKey(key.into()))
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
        self.inner
            .put(CacheBucketKey(key.into()), bytes.as_ref().to_vec())
            .execute()
            .into_send()
            .await
            .map(|_obj| ())
    }
}

impl ObjectGateway for CacheBucket {
    type Storage = Self;

    fn object_url(&self, access_request: StorageAccessRequest<'_>) -> url::Url {
        // R2 doesn't support unauthenticated writes, so we authenticate
        // via a presigned URL. R2 only supports presigned URLs on the
        // S3-compatible endpoint, not on custom domains (see
        // https://developers.cloudflare.com/r2/api/s3/presigned-urls/#custom-domains).
        //
        // TODO: switch to authenticating via WAF HMAC validation once it makes
        // sense to start paying for a Cloudflare Pro plan.
        if access_request.method == http::Method::PUT {
            let url = S3_ENDPOINT
                .join(&CacheBucketKey(access_request.key).to_string())
                .expect("combined URL is valid");

            let now = chrono::DateTime::from_timestamp_millis(
                js_sys::Date::now() as i64,
            )
            .expect("current time is a valid timestamp");

            let creds = self.s3_creds();

            s3v4::pre_signed_url(
                &creds.access_key_id,
                &creds.secret_access_key,
                access_request.min_ttl.as_secs(),
                &url,
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
            PUBLIC_ENDPOINT
                .join(&CacheBucketKey(access_request.key).to_string())
                .expect("combined URL is valid")
        }
    }

    fn storage(&self) -> &Self {
        self
    }
}

impl fmt::Display for CacheBucketKey<'_> {
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

impl From<CacheBucketKey<'_>> for String {
    fn from(key: CacheBucketKey<'_>) -> Self {
        key.to_string()
    }
}
