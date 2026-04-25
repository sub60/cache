use core::fmt;
use std::sync::{LazyLock, OnceLock};

use axum::http;
use cache::object_gateway::{MultipartUploadUrlRequest, ObjectUrlRequest};
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
    type MultipartUploadId = String;
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

    async fn create_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Self::MultipartUploadId, Self::Error> {
        Ok(self
            .inner
            .create_multipart_upload(CacheBucketKey(key.into()))
            .execute()
            .into_send()
            .await?
            .upload_id()
            .await)
    }
}

impl ObjectGateway for CacheBucket {
    type Storage = Self;

    fn object_url(&self, access_request: ObjectUrlRequest<'_>) -> url::Url {
        // Writes go through the S3-compatible endpoint and are authenticated
        // via a presigned URL. The S3 endpoint also bypasses Cloudflare's edge
        // proxy, which would otherwise enforce a 100MiB body size limit
        // (there's still a 5GiB limit on single-part uploads though).
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
        }
        // Reads go through our public R2 domain, which is fronted by
        // Cloudflare's edge proxy and gives us CDN caching.
        else {
            PUBLIC_ENDPOINT
                .join(&CacheBucketKey(access_request.key).to_string())
                .expect("combined URL is valid")
        }
    }

    fn multipart_upload_url(
        &self,
        request: MultipartUploadUrlRequest<'_, String>,
    ) -> url::Url {
        let mut url = S3_ENDPOINT
            .join(&CacheBucketKey(request.key).to_string())
            .expect("combined URL is valid");

        url.query_pairs_mut()
            .append_pair("partNumber", &request.part_number.to_string())
            .append_pair("uploadId", request.upload_id);

        let now =
            chrono::DateTime::from_timestamp_millis(js_sys::Date::now() as i64)
                .expect("current time is a valid timestamp");

        let creds = self.s3_creds();

        s3v4::pre_signed_url(
            &creds.access_key_id,
            &creds.secret_access_key,
            request.min_ttl.as_secs(),
            &url,
            http::Method::PUT.as_str(),
            "UNSIGNED-PAYLOAD",
            "auto",
            &now,
            "s3",
        )
        .expect("presigning should not fail")
        .parse()
        .expect("presigned URL is a valid URI")
    }

    fn storage(&self) -> &Self {
        self
    }
}

impl fmt::Display for CacheBucketKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ObjectStorageKey::Nar(nar) => write!(f, "nar/{nar}"),
            ObjectStorageKey::NarMultipart(upload_id) => {
                write!(f, "nar-multipart/{upload_id}")
            },
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
