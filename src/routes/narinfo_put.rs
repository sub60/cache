//! TODO: docs.

use core::fmt;
use core::str::FromStr;

use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::ObjectStorage;
use crate::context::Context;
use crate::types::{
    CompressionAlgorithm,
    NarFileName,
    NarFileNameFromStrError,
    NarInfo,
    NarInfoFromStrError,
    Nix32Hash,
    NixStoreHash,
};

pub(crate) const ROUTE: &str = crate::routes::narinfo_get::ROUTE;

/// The response returned to PUT requests to the NARInfo route.
#[derive(Debug, Copy, Clone)]
#[expect(dead_code)]
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
        declared_file_hash: Nix32Hash<32>,
        /// The file hash inferred from `URL` field.
        inferred_file_hash: Nix32Hash<32>,
    },

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
    routing::put(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_store_hash): Path<NixStoreHash>,
    _ctx: &mut Ctx,
) -> Result<
    NarInfoPutResponse,
    NarInfoPutRequestError<<Ctx::ObjectStorage as ObjectStorage>::Error>,
> {
    todo!()
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
        match *self {
            Self::NarCompressionMismatch { .. }
            | Self::NarFileHashMismatch { .. }
            | Self::NarInfoStoreHashMismatch { .. }
            | Self::NarNotFound(_)
            | Self::ParseNarInfo(_) => StatusCode::BAD_REQUEST,
            Self::CheckNarExistence(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
