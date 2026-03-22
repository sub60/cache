use core::fmt;
use core::time::Duration;

use axum::http;
use cache::object_gateway::StorageAccessRequest;
use cache::object_storage::ObjectStorageKey;
use cache::{ObjectGateway, ObjectStorage};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
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

    /// Returns the host of the S3-compatible endpoint for the given account ID.
    fn s3_endpoint_host(account_id: &str) -> impl AsRef<str> {
        format!("{account_id}.r2.cloudflarestorage.com")
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
            let base_url: http::Uri = format!(
                "https://{}/{}/{}",
                Self::s3_endpoint_host(&self.s3_creds.account_id).as_ref(),
                self.s3_creds.bucket_name,
                R2BucketKey(access_request.key),
            )
            .parse()
            .expect("valid S3 URI");

            presigned_url(
                &base_url,
                &self.s3_creds,
                &access_request.method,
                access_request.ttl.unwrap_or(Duration::from_secs(3600)),
            )
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

/// Signs a base URL with AWS Signature Version 4, returning a presigned URL
/// with the appropriate authentication query parameters.
fn presigned_url(
    base_url: &http::Uri,
    creds: &R2S3Credentials,
    method: &http::Method,
    ttl: Duration,
) -> http::Uri {
    let host = base_url.host().expect("base URL must have a host");
    let uri_path = base_url.path();

    let (amz_date, date_stamp) = current_sigv4_timestamps();

    let credential_scope = format!("{date_stamp}/auto/s3/aws4_request");
    let credential = format!("{}/{credential_scope}", creds.access_key_id);

    let canonical_query_string =
        sigv4_canonical_query_string(&credential, &amz_date, ttl);

    let canonical_request = format!(
        "{}\n{}\n{canonical_query_string}\nhost:{host}\n\nhost\\
         nUNSIGNED-PAYLOAD",
        method.as_str(),
        uri_encode_path(uri_path),
    );

    let string_to_sign =
        sigv4_string_to_sign(&amz_date, &credential_scope, &canonical_request);

    let signing_key = sigv4_signing_key(&creds.secret_access_key, &date_stamp);

    let signature = hex::encode(hmac_sha256(&signing_key, &string_to_sign));

    format!("{base_url}?{canonical_query_string}&X-Amz-Signature={signature}",)
        .parse()
        .expect("presigned URL is a valid URI")
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
    ttl: Duration,
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
        format!("X-Amz-Expires={}", ttl.as_secs()),
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

/// URI-encodes a string per RFC 3986, encoding everything except unreserved
/// characters (A-Z, a-z, 0-9, '-', '.', '_', '~').
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
