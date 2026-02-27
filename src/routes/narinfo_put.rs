//! TODO: docs.

use core::str::FromStr;
use core::{fmt, str};

use axum::body::{Body, Bytes};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::ObjectStorageError;
use crate::types::{
    CompressionAlgorithm,
    NarFileName,
    NarFileNameFromStrError,
    NarInfo,
    NarInfoFileName,
    NarInfoFileNameFromStrError,
    NarInfoFromStrError,
    Nix32Digest,
    NixStoreHash,
};
use crate::{Context, ObjectStorage};

pub(crate) const ROUTE: &str = crate::routes::narinfo_get::ROUTE;

/// The response returned to PUT requests to the NARInfo route.
#[derive(Debug, Copy, Clone)]
enum NarInfoPutResponse {
    /// The NARInfo already exists in the user's cache, so the request was a
    /// no-op.
    AlreadyExists,
    /// The NARInfo was successfully created.
    Created,
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

    /// The compression algorithm declared by the client in the `Compression`
    /// field of the NARInfo did not match the compression algorithm inferred
    /// from the NAR file name in the URL.
    #[display(
        "compression algorithm mismatch: declared '{declared_compression}' in
         'Compression', inferred '{inferred_compression}' from filename
         extension in 'URL'"
    )]
    NarCompressionMismatch {
        /// The compression algorithm in the `Compression` field.
        declared_compression: CompressionAlgorithm,
        /// The compression algorithm inferred from `URL` field.
        inferred_compression: CompressionAlgorithm,
    },

    /// The NAR hash declared by the client in the `FileHash` field of the
    /// NARInfo did not match the hash inferred from the NAR file name in the
    /// URL.
    #[display(
        "NAR hash mismatch: declared '{declared_file_hash}' in 'FileHash', \
         inferred '{inferred_file_hash}' from filename in 'URL'"
    )]
    NarFileHashMismatch {
        /// The file hash the `FileHash` field.
        declared_file_hash: Nix32Digest<32>,
        /// The file hash inferred from `URL` field.
        inferred_file_hash: Nix32Digest<32>,
    },

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
        declared_store_hash: NixStoreHash,
        /// The store hash inferred from the `StorePath` field of the NARInfo.
        inferred_store_hash: NixStoreHash,
    },

    /// The NAR file was not found in the object storage.
    #[display("NARInfo at '{_0}' not found")]
    NarNotFound(NarFileName),

    /// The NARInfo we got from the client did not parse into a valid
    /// [`NarInfo`].
    ParseNarInfo(#[from] NarInfoFromStrError<<NarPutUrl as FromStr>::Err>),

    /// The object storage returned an error when storing the NARInfo.
    StoreNarInfo(ObjectStorageError),
}

/// A struct whose [`FromStr`] implementation parses strings of the form
/// `nar/{nar_file_name}`, which is the format used by the Nix client when
/// sending a PUT request to the NARInfo route.
struct NarPutUrl {
    nar_file_name: NarFileName,
}

/// The type of error that can occur when parsing a string into a [`NarPutUrl`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarPutUrlFromStrError {
    /// The input string did not start with the expected "nar/" prefix.
    #[display("missing expected \"nar/\" prefix")]
    MissingNarPrefix,
    /// The NAR file name part of the input string did not parse successfully.
    #[display("{_0}")]
    InvalidNarFileName(NarFileNameFromStrError),
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::put(async move |path, body| handler(path, body, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(narinfo_file_name): Path<NarInfoFileName>,
    body: Bytes,
    ctx: &mut Ctx,
) -> Result<NarInfoPutResponse, NarInfoPutRequestError<ObjectStorageError<Ctx>>>
{
    // TODO: validate that the user has write permissions for that cache.

    let nar_info = str::from_utf8(&body)
        .map_err(NarInfoPutRequestError::NarInfoNotUtf8)?
        .parse::<NarInfo<NarPutUrl>>()?;

    if &narinfo_file_name.store_hash != nar_info.store_path.hash() {
        return Err(NarInfoPutRequestError::NarInfoStoreHashMismatch {
            declared_store_hash: narinfo_file_name.store_hash,
            inferred_store_hash: nar_info.store_path.into_hash(),
        });
    }

    let inferred_compression = nar_info
        .url
        .nar_file_name
        .extension
        .map(|ext| ext.algorithm())
        .unwrap_or(CompressionAlgorithm::None);

    if nar_info.compression != inferred_compression {
        return Err(NarInfoPutRequestError::NarCompressionMismatch {
            declared_compression: nar_info.compression,
            inferred_compression,
        });
    }

    if nar_info.file_hash != nar_info.url.nar_file_name.file_hash {
        return Err(NarInfoPutRequestError::NarFileHashMismatch {
            declared_file_hash: nar_info.file_hash,
            inferred_file_hash: nar_info.url.nar_file_name.file_hash,
        });
    }

    let object_storage = ctx.object_storage();

    if !object_storage
        .contains_key(&(&nar_info.url.nar_file_name).into())
        .await
        .map_err(NarInfoPutRequestError::CheckNarExistence)?
    {
        return Err(NarInfoPutRequestError::NarNotFound(
            nar_info.url.nar_file_name,
        ));
    }

    let stored_nar_info = nar_info.with_url("");

    let nar_info_was_inserted = object_storage
        .set(
            (&narinfo_file_name).into(),
            stored_nar_info.to_string().as_bytes(),
        )
        .await
        .map_err(NarInfoPutRequestError::StoreNarInfo)?;

    if nar_info_was_inserted {
        Ok(NarInfoPutResponse::Created)
    } else {
        Ok(NarInfoPutResponse::AlreadyExists)
    }
}

impl NarInfoPutResponse {
    fn status_code(self) -> StatusCode {
        match self {
            Self::AlreadyExists => StatusCode::NO_CONTENT,
            Self::Created => StatusCode::CREATED,
        }
    }
}

impl<ObjectStorageError> NarInfoPutRequestError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidNarInfoFileName(_)
            | Self::NarCompressionMismatch { .. }
            | Self::NarFileHashMismatch { .. }
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

impl IntoResponse for NarInfoPutResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Body::empty()).into_response()
    }
}

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarInfoPutRequestError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl FromStr for NarPutUrl {
    type Err = NarPutUrlFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix("nar/")
            .ok_or(NarPutUrlFromStrError::MissingNarPrefix)?
            .parse()
            .map_err(NarPutUrlFromStrError::InvalidNarFileName)
            .map(|nar_file_name| Self { nar_file_name })
    }
}
