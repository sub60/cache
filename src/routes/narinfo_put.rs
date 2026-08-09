//! TODO: docs.

use core::convert::Infallible;
use core::str::FromStr;
use core::{fmt, str};

use axum::Extension;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use nix_types::{
    NarFileName,
    NarInfo,
    NarInfoFileName,
    NarInfoFileNameFromStrError,
    NarInfoFromStrError,
    StoreHash,
};

use crate::context::ObjectStorageError;
use crate::object_storage::ObjectStorageKey;
use crate::{CacheDb, CacheInfos, Context, ObjectStorage};

pub(crate) const ROUTE: &str = crate::routes::narinfo_get::ROUTE;

#[derive(serde::Deserialize)]
struct NarInfoPutPath {
    narinfo_file_name: NarInfoFileName,
}

/// The type of error that can occur when handling a PUT request to the NARInfo
/// route.
#[derive(
    Debug, PartialEq, Eq, derive_more::Display, cauchy::Error, cauchy::From,
)]
enum NarInfoPutRequestError<ObjectStorageError> {
    /// The requested NARInfo file name in the path was invalid.
    #[display("{_0}")]
    InvalidNarInfoFileName(#[from] NarInfoFileNameFromStrError),

    /// The object storage returned an error when checking for the existence of
    /// the NAR file.
    #[display("error checking for existence of NAR: {_0}")]
    CheckNarExistence(ObjectStorageError),

    /// The NARInfo in the body of the request was not valid UTF-8.
    #[display("NARInfo in request body is not valid UTF-8: {_0}")]
    NarInfoNotUtf8(str::Utf8Error),

    /// The NARInfo store hash declared by the client in the URL of the request
    /// did not match the hash in the `StorePath` field of the NARInfo.
    #[display(
        "NARInfo store hash mismatch: declared '{declared_store_hash}' in \
         request URL, inferred {inferred_store_hash} from hash in 'StorePath'"
    )]
    NarInfoStoreHashMismatch {
        /// The store hash in the request URL.
        declared_store_hash: StoreHash,
        /// The store hash inferred from the `StorePath` field of the NARInfo.
        inferred_store_hash: StoreHash,
    },

    /// The NAR file was not found in the object storage.
    #[display("NAR at '{_0}' not found")]
    NarNotFound(NarFileName),

    /// The NARInfo we got from the client did not parse into a valid
    /// [`NarInfo`].
    ParseNarInfo(#[from] NarInfoFromStrError<<DiscardUrl as FromStr>::Err>),

    /// The object storage returned an error when storing the NARInfo.
    StoreNarInfo(ObjectStorageError),
}

/// A type whose [`FromStr`] impl discards the string, used to ignore the `URL`
/// field in the client-provided NARInfo.
struct DiscardUrl;

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::put(async move |Path(path), Extension(cache_infos), body| {
        handler(path, cache_infos, body, &mut ctx).await
    })
}

async fn handler<Ctx: Context>(
    NarInfoPutPath { narinfo_file_name }: NarInfoPutPath,
    cache_infos: CacheInfos<<Ctx::CacheDb as CacheDb>::CacheId>,
    body: Bytes,
    ctx: &mut Ctx,
) -> Result<StatusCode, NarInfoPutRequestError<ObjectStorageError<Ctx>>> {
    // TODO: validate that the user has write permissions for that cache.
    // TODO: validate that the store dir in the NARInfo's StorePath matches the
    // cache's storeDir.

    let mut narinfo = str::from_utf8(&body)
        .map_err(NarInfoPutRequestError::NarInfoNotUtf8)?
        .parse::<NarInfo<DiscardUrl>>()?;

    if &narinfo_file_name.store_hash != narinfo.store_path.hash() {
        return Err(NarInfoPutRequestError::NarInfoStoreHashMismatch {
            declared_store_hash: narinfo_file_name.store_hash,
            inferred_store_hash: narinfo.store_path.into_hash(),
        });
    }

    let object_storage = ctx.object_storage();

    let nar_file_name = NarFileName::from(&narinfo);

    if !object_storage
        .contains_key(ObjectStorageKey::Nar(&nar_file_name))
        .await
        .map_err(NarInfoPutRequestError::CheckNarExistence)?
    {
        return Err(NarInfoPutRequestError::NarNotFound(nar_file_name));
    }

    // Sign the fingerprint with our private key and append the signature to the
    // NARInfo before storing it.
    let cache_signature = cache_infos
        .signing_key
        .sign(narinfo.fingerprint().to_string().as_bytes());

    narinfo.signatures.push(cache_signature);

    match object_storage.insert(&narinfo_file_name, narinfo.to_string()).await {
        Ok(true) => Ok(StatusCode::CREATED),
        Ok(false) => Ok(StatusCode::OK),
        Err(err) => Err(NarInfoPutRequestError::StoreNarInfo(err)),
    }
}

impl<ObjectStorageError> NarInfoPutRequestError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidNarInfoFileName(_)
            | Self::NarInfoNotUtf8(_)
            | Self::NarInfoStoreHashMismatch { .. }
            | Self::NarNotFound(_)
            | Self::ParseNarInfo(_) => StatusCode::BAD_REQUEST,
            Self::CheckNarExistence(_) | Self::StoreNarInfo(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }
}

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarInfoPutRequestError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl FromStr for DiscardUrl {
    type Err = Infallible;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

impl fmt::Display for DiscardUrl {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
