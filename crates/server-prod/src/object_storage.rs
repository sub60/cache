use core::fmt;
use core::time::Duration;

use axum::http;
use cache::object_gateway::PresignRequest;
use cache::object_storage::ObjectStorageKey;
use cache::{ObjectGateway, ObjectStorage};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use types::NarFileName;
use worker::send::IntoSendFuture;

/// The [`ObjectStorage`] and [`ObjectGateway`] implementation backed by
/// Cloudflare R2.
#[derive(Debug, Clone)]
pub struct R2ObjectStorage {
    bucket: worker::Bucket,
    s3_creds: R2S3Credentials,
}

/// S3-compatible credentials for generating presigned R2 URLs.
///
/// These are obtained by creating an R2 API token in the Cloudflare
/// dashboard. The token must have "Object Read & Write" permission scoped
/// to the target bucket.
#[derive(Debug, Clone)]
pub(crate) struct R2S3Credentials {
    /// The Cloudflare account ID (used to construct the S3-compatible
    /// endpoint).
    pub account_id: String,
    /// The R2 API token's Access Key ID.
    pub access_key_id: String,
    /// The R2 API token's Secret Access Key.
    pub secret_access_key: String,
    /// The name of the R2 bucket.
    pub bucket_name: String,
}

struct R2BucketKey<'a>(ObjectStorageKey<'a>);

impl R2ObjectStorage {
    /// Creates a new [`R2ObjectStorage`] backed by the given R2 bucket.
    pub fn new(bucket: worker::Bucket, s3_creds: R2S3Credentials) -> Self {
        Self { bucket, s3_creds }
    }
}

impl R2S3Credentials {
    /// Generates a presigned URL using AWS Signature Version 4.
    fn presign_url(
        &self,
        request: PresignRequest<ObjectStorageKey<'_>>,
    ) -> http::Uri {
        let object_key = R2BucketKey(request.key).to_string();
        let host = format!("{}.r2.cloudflarestorage.com", self.account_id);
        let uri_path = format!("/{}/{object_key}", self.bucket_name);
        let expires_in =
            request.ttl.unwrap_or(Duration::from_secs(3600)).as_secs();
        let method = request.method.as_str();

        let (amz_date, date_stamp) = current_sigv4_timestamps();

        let credential_scope = format!("{date_stamp}/auto/s3/aws4_request");
        let credential = format!("{}/{credential_scope}", self.access_key_id);

        let canonical_query_string =
            sigv4_canonical_query_string(&credential, &amz_date, expires_in);

        let canonical_request = format!(
            "{method}\n{}\n{canonical_query_string}\nhost:{host}\n\nhost\\
             nUNSIGNED-PAYLOAD",
            uri_encode_path(&uri_path),
        );

        let string_to_sign = sigv4_string_to_sign(
            &amz_date,
            &credential_scope,
            &canonical_request,
        );

        let signing_key =
            sigv4_signing_key(&self.secret_access_key, &date_stamp);

        let signature = hex::encode(hmac_sha256(&signing_key, &string_to_sign));

        format!(
            "https://{host}{uri_path}?\
             {canonical_query_string}&X-Amz-Signature={signature}",
        )
        .parse()
        .expect("presigned URL is a valid URI")
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

    fn nar_url<const IS_FOR_PUT: bool>(
        &self,
        filename: &NarFileName,
    ) -> http::Uri {
        // R2 only supports presigned URLs on the S3-compatible endpoint, not on
        // custom domains (see https://developers.cloudflare.com/r2/api/s3/presigned-urls/#custom-domains).
        //
        // TODO: switch to using WAF HMAC validation once it makes sense to
        // start paying for a Cloudflare Pro plan.
        let base = if IS_FOR_PUT {
            format!(
                "https://{}.r2.cloudflarestorage.com/{}",
                self.s3_creds.account_id, self.s3_creds.bucket_name,
            )
        } else {
            String::new()
        };

        format!("{base}/{}", R2BucketKey(ObjectStorageKey::Nar(filename)))
            .parse()
            .expect("valid URI")
    }

    fn storage(&self) -> &Self {
        self
    }

    fn presign(
        &self,
        presign_request: PresignRequest<ObjectStorageKey<'_>>,
    ) -> http::Uri {
        self.s3_creds.presign_url(presign_request)
    }
}

/// Returns `(amz_date, date_stamp)` for the current UTC time.
///
/// `amz_date` is `YYYYMMDDTHHMMSSZ`, `date_stamp` is `YYYYMMDD`.
fn current_sigv4_timestamps() -> (String, String) {
    let now = js_sys::Date::new_0();
    let iso: String = now.to_iso_string().into();
    let amz_date = format!(
        "{}T{}Z",
        &iso[..10].replace('-', ""),
        &iso[11..19].replace(':', ""),
    );
    let date_stamp = amz_date[..8].to_owned();
    (amz_date, date_stamp)
}

/// Builds the SigV4 canonical query string (already sorted).
fn sigv4_canonical_query_string(
    credential: &str,
    amz_date: &str,
    expires_in: u64,
) -> String {
    // These are pre-sorted by key name (X-Amz-A < X-Amz-C < X-Amz-D
    // < X-Amz-E < X-Amz-S).
    [
        ("X-Amz-Algorithm", "AWS4-HMAC-SHA256"),
        ("X-Amz-Content-Sha256", "UNSIGNED-PAYLOAD"),
        ("X-Amz-Credential", credential),
        ("X-Amz-Date", amz_date),
    ]
    .into_iter()
    .map(|(k, v)| format!("{k}={}", uri_encode(v)))
    .chain([
        format!("X-Amz-Expires={expires_in}"),
        "X-Amz-SignedHeaders=host".to_owned(),
    ])
    .collect::<Vec<_>>()
    .join("&")
}

/// Builds the SigV4 string to sign from the canonical request.
fn sigv4_string_to_sign(
    amz_date: &str,
    credential_scope: &str,
    canonical_request: &str,
) -> String {
    let hashed = hex::encode(Sha256::digest(canonical_request.as_bytes()));
    format!("AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{hashed}",)
}

/// URI-encodes a string per RFC 3986, encoding everything except
/// unreserved characters (A-Z, a-z, 0-9, '-', '.', '_', '~').
fn uri_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~' => encoded.push(byte as char),
            _ => {
                encoded.push('%');
                encoded.push_str(&hex::encode([byte]).to_uppercase());
            },
        }
    }
    encoded
}

/// URI-encodes a path, preserving `/` separators.
fn uri_encode_path(path: &str) -> String {
    path.split('/').map(uri_encode).collect::<Vec<_>>().join("/")
}

/// Computes HMAC-SHA256.
fn hmac_sha256(key: &[u8], msg: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("HMAC accepts any key length");
    mac.update(msg.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// Derives the SigV4 signing key for the `auto` region and `s3` service.
fn sigv4_signing_key(secret_key: &str, date_stamp: &str) -> Vec<u8> {
    let k_date =
        hmac_sha256(format!("AWS4{secret_key}").as_bytes(), date_stamp);
    let k_region = hmac_sha256(&k_date, "auto");
    let k_service = hmac_sha256(&k_region, "s3");
    hmac_sha256(&k_service, "aws4_request")
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
