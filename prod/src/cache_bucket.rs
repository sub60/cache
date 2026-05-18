use core::fmt;
use core::num::NonZeroU32;
use core::time::Duration;
use std::sync::{LazyLock, OnceLock};
use std::time::SystemTime;

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    PayloadChecksumKind,
    PercentEncodingMode,
    SignableBody,
    SignableRequest,
    SignatureLocation,
    SigningInstructions,
    SigningSettings,
    UriPathNormalizationMode,
};
use aws_sigv4::sign::v4;
use axum::http;
use nix_types::sub60::nar_multiparts::Etag;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sub60_cache::{
    MultipartUploadUrlRequest,
    ObjectGateway,
    ObjectStorage,
    ObjectStorageKey,
    ObjectUrlRequest,
};
use worker::HttpMetadata;
use worker::send::IntoSendFuture;

const MAX_MULTIPART_PARTS: usize = 10_000;

const NAR_CONTENT_TYPE: &str = "application/x-nix-nar";

const NARINFO_CONTENT_TYPE: &str = "text/x-nix-narinfo";

/// 16 MiB.
const PART_SIZE: NonZeroU32 =
    NonZeroU32::new(16 * 1024 * 1024).expect("multipart part size is non-zero");

const S3_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'/')
    .remove(b'_')
    .remove(b'~');

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

#[derive(Copy, Clone)]
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

    async fn copy<'src, 'dst>(
        &self,
        src: CacheBucketKey<'src>,
        dest: CacheBucketKey<'dst>,
    ) -> Result<(), worker::Error> {
        let copy_source =
            format!("/{}/{}", env!("R2_BUCKET_NAME"), src.percent_encode());

        let object_url = dest.s3_url();

        let signing_instructions =
            sign_s3_copy_request(self.s3_creds(), &object_url, &copy_source);

        let headers = worker::Headers::new();
        headers.set("x-amz-copy-source", &copy_source)?;
        for (name, value) in signing_instructions.headers() {
            headers.set(name, value)?;
        }

        let request = {
            let mut init = worker::RequestInit::new();
            init.with_method(worker::Method::Put).with_headers(headers);
            worker::Request::new_with_init(object_url.as_str(), &init)?
        };

        let mut response =
            worker::Fetch::Request(request).send().into_send().await?;

        let status_code = response.status_code();

        if (200..300).contains(&status_code) {
            Ok(())
        } else {
            let body =
                response.text().into_send().await.unwrap_or_else(|err| {
                    format!("<failed to read response body: {err}>")
                });

            Err(worker::Error::RustError(format!(
                "R2 CopyObject failed with status {status_code}: {body}"
            )))
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

impl CacheBucketKey<'_> {
    fn content_type(self) -> &'static str {
        match self.0 {
            ObjectStorageKey::Nar(_) | ObjectStorageKey::NarMultipart(_) => {
                NAR_CONTENT_TYPE
            },
            ObjectStorageKey::NarInfo(_) => NARINFO_CONTENT_TYPE,
        }
    }

    fn http_metadata(self) -> HttpMetadata {
        HttpMetadata {
            content_type: Some(self.content_type().to_owned()),
            ..Default::default()
        }
    }

    fn percent_encode(self) -> String {
        let key_str = self.to_string();

        match self.0 {
            // Keys for NARs and NARInfos are guaranteed to only contain
            // URL-safe characters, so we don't need to encode it further.
            ObjectStorageKey::Nar(_) | ObjectStorageKey::NarInfo(_) => key_str,
            // Multipart NARs are keyed by store basename, and the name part in
            // the basename can contain arbitrary characters, so we need to
            // percent-encode it.
            ObjectStorageKey::NarMultipart(_) => {
                utf8_percent_encode(&key_str, S3_ENCODE_SET).to_string()
            },
        }
    }

    fn s3_url(self) -> url::Url {
        S3_ENDPOINT.join(&self.percent_encode()).expect("combined URL is valid")
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
        let bucket_key = CacheBucketKey(key.into());

        self.inner
            .put(bucket_key, bytes.as_ref().to_vec())
            .http_metadata(bucket_key.http_metadata())
            .execute()
            .into_send()
            .await
            .map(|_obj| ())
    }

    async fn r#move<'src, 'dst>(
        &self,
        source: impl Into<ObjectStorageKey<'src>> + Send,
        destination: impl Into<ObjectStorageKey<'dst>> + Send,
    ) -> Result<(), Self::Error> {
        let src = CacheBucketKey(source.into());
        let dest = CacheBucketKey(destination.into());
        self.copy(src, dest).await?;
        self.inner.delete(src).into_send().await
    }

    async fn create_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Self::MultipartUploadId, Self::Error> {
        let bucket_key = CacheBucketKey(key.into());

        Ok(self
            .inner
            .create_multipart_upload(bucket_key)
            .http_metadata(bucket_key.http_metadata())
            .execute()
            .into_send()
            .await?
            .upload_id()
            .await)
    }

    async fn complete_multipart_upload<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
        upload_id: Self::MultipartUploadId,
        etags: &[Etag],
    ) -> Result<(), Self::Error> {
        let uploaded_parts =
            etags.iter().enumerate().map(|(part_idx, etag)| {
                // The caller guarantees etags to contain <= MAX_MULTIPART_PARTS
                // elements, so we can directly cast the part_idx into a u16.
                let part_number = part_idx as u16 + 1;
                worker::UploadedPart::new(part_number, etag.to_string())
            });

        self.inner
            .resume_multipart_upload(CacheBucketKey(key.into()), upload_id)?
            .complete(uploaded_parts)
            .into_send()
            .await
            .map(|_object| ())
    }

    fn max_multipart_parts(&self) -> Option<usize> {
        Some(MAX_MULTIPART_PARTS)
    }

    fn part_size(&self) -> NonZeroU32 {
        PART_SIZE
    }
}

fn sign_s3_copy_request(
    credentials: &CacheBucketS3Credentials,
    url: &url::Url,
    copy_source: &str,
) -> SigningInstructions {
    let identity = Credentials::new(
        &credentials.access_key_id,
        &credentials.secret_access_key,
        None,
        None,
        "cache-bucket",
    )
    .into();

    let mut signing_settings = SigningSettings::default();
    signing_settings.payload_checksum_kind = PayloadChecksumKind::XAmzSha256;
    signing_settings.percent_encoding_mode = PercentEncodingMode::Single;
    signing_settings.uri_path_normalization_mode =
        UriPathNormalizationMode::Disabled;

    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region("auto")
        .name("s3")
        .time(
            // SystemTime::now() panics on WASM.
            SystemTime::UNIX_EPOCH
                + Duration::from_millis(js_sys::Date::now() as u64),
        )
        .settings(signing_settings)
        .build()
        .expect("R2 CopyObject signing params are fully specified")
        .into();

    let headers = [("x-amz-copy-source", copy_source)];
    let signable_request = SignableRequest::new(
        http::Method::PUT.as_str(),
        url.as_str(),
        headers.into_iter(),
        SignableBody::Bytes(&[]),
    )
    .expect("R2 CopyObject URL is a valid signable URI");

    aws_sigv4::http_request::sign(signable_request, &signing_params)
        .map(|output| output.into_parts().0)
        .expect("R2 CopyObject request is signable")
}

impl ObjectGateway for CacheBucket {
    type Storage = Self;

    fn object_url(&self, access_request: ObjectUrlRequest<'_>) -> url::Url {
        let bucket_key = CacheBucketKey(access_request.key);

        // Writes go through the S3-compatible endpoint and are authenticated
        // via a presigned URL. The S3 endpoint also bypasses Cloudflare's edge
        // proxy, which would otherwise enforce a 100MiB body size limit
        // (there's still a 5GiB limit on single-part uploads though).
        if access_request.method == http::Method::PUT {
            sign_s3_presigned_url(
                self.s3_creds(),
                &bucket_key.s3_url(),
                &access_request.method,
                access_request.min_ttl,
                &[(
                    http::header::CONTENT_TYPE.as_str(),
                    bucket_key.content_type(),
                )],
            )
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
        let mut url = CacheBucketKey(request.key).s3_url();

        url.query_pairs_mut()
            .append_pair("partNumber", &request.part_number.to_string())
            .append_pair("uploadId", request.upload_id);

        sign_s3_presigned_url(
            self.s3_creds(),
            &url,
            &http::Method::PUT,
            request.min_ttl,
            &[],
        )
    }

    fn storage(&self) -> &Self {
        self
    }
}

#[expect(clippy::too_many_arguments)]
fn sign_s3_presigned_url(
    credentials: &CacheBucketS3Credentials,
    url: &url::Url,
    method: &http::Method,
    min_ttl: Duration,
    headers: &[(&str, &str)],
) -> url::Url {
    let identity = Credentials::new(
        &credentials.access_key_id,
        &credentials.secret_access_key,
        None,
        None,
        "cache-bucket",
    )
    .into();

    let mut signing_settings = SigningSettings::default();
    signing_settings.signature_location = SignatureLocation::QueryParams;
    signing_settings.expires_in = Some(min_ttl);
    signing_settings.percent_encoding_mode = PercentEncodingMode::Single;
    signing_settings.uri_path_normalization_mode =
        UriPathNormalizationMode::Disabled;

    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region("auto")
        .name("s3")
        .time(
            // SystemTime::now() panics on WASM.
            SystemTime::UNIX_EPOCH
                + Duration::from_millis(js_sys::Date::now() as u64),
        )
        .settings(signing_settings)
        .build()
        .expect("S3 presigning params are fully specified")
        .into();

    let signable_request = SignableRequest::new(
        method.as_str(),
        url.as_str(),
        headers.iter().copied(),
        SignableBody::UnsignedPayload,
    )
    .expect("S3 presigned URL is a valid signable URI");

    let (signing_instructions, _signature) =
        aws_sigv4::http_request::sign(signable_request, &signing_params)
            .map(|output| output.into_parts())
            .expect("S3 presigned URL is signable");

    let mut request = http::Request::builder()
        .method(method)
        .uri(url.as_str())
        .body(())
        .expect("S3 presigned URL request is valid");

    signing_instructions.apply_to_request_http1x(&mut request);

    request.uri().to_string().parse().expect("S3 presigned URL is valid")
}

impl fmt::Display for CacheBucketKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ObjectStorageKey::Nar(nar) => write!(f, "nar/{nar}"),
            ObjectStorageKey::NarMultipart(store_basename) => {
                write!(f, "nar-multipart/{store_basename}")
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
