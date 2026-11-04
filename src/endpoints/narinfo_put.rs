//! TODO: docs.

use core::convert::Infallible;
use core::str::FromStr;
use core::{fmt, str};

use axum::Extension;
use axum::body::Bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use nix_types::{
    NarFileName,
    NarInfo,
    NarInfoFileNameFromStrError,
    NarInfoFromStrError,
    StoreHash,
};

use crate::api::endpoints::narinfo_put;
use crate::context::{NarInfoStorageError, NarStorageError};
use crate::{
    CacheInfos,
    Context,
    NarInfoStorage,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

/// The type of error that can occur when handling a PUT request to the NARInfo
/// route.
#[derive(
    Debug, PartialEq, Eq, derive_more::Display, cauchy::Error, cauchy::From,
)]
enum NarInfoPutRequestError<NarStorageError, NarInfoStorageError> {
    /// The requested NARInfo file name in the path was invalid.
    #[display("{_0}")]
    InvalidNarInfoFileName(#[from] NarInfoFileNameFromStrError),

    /// The NAR storage returned an error when checking for the existence of
    /// the NAR file.
    #[display("error checking for existence of NAR: {_0}")]
    CheckNarExistence(NarStorageError),

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

    /// The NAR file was not found in the NAR storage.
    #[display("NAR at '{_0}' not found")]
    NarNotFound(NarFileName),

    /// The NARInfo we got from the client did not parse into a valid
    /// [`NarInfo`].
    ParseNarInfo(#[from] NarInfoFromStrError<<DiscardUrl as FromStr>::Err>),

    /// The NARInfo storage returned an error when storing the NARInfo.
    StoreNarInfo(NarInfoStorageError),
}

/// A type whose [`FromStr`] impl discards the string, used to ignore the `URL`
/// field in the client-provided NARInfo.
struct DiscardUrl;

impl endpoints::CacheEndpoint for narinfo_put::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PushToCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |path, Extension(cache_infos), body| {
                handler(path, cache_infos, body, &mut ctx)
                    .await
                    .map(|response| response.status_code())
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: narinfo_put::Path,
    cache_infos: CacheInfos,
    body: Bytes,
    ctx: &mut Ctx,
) -> Result<
    narinfo_put::Response,
    NarInfoPutRequestError<NarStorageError<Ctx>, NarInfoStorageError<Ctx>>,
> {
    // TODO: validate that the store dir in the NARInfo's StorePath matches the
    // cache's storeDir.

    let narinfo = str::from_utf8(&body)
        .map_err(NarInfoPutRequestError::NarInfoNotUtf8)?
        .parse::<NarInfo<DiscardUrl>>()?;

    if &path.narinfo_file_name.store_hash != narinfo.store_path.hash() {
        return Err(NarInfoPutRequestError::NarInfoStoreHashMismatch {
            declared_store_hash: path.narinfo_file_name.store_hash,
            inferred_store_hash: narinfo.store_path.into_hash(),
        });
    }

    let nar_file_name = NarFileName::from(&narinfo);

    if !ctx
        .nar_storage()
        .contains(&cache_infos, &nar_file_name)
        .await
        .map_err(NarInfoPutRequestError::CheckNarExistence)?
    {
        return Err(NarInfoPutRequestError::NarNotFound(nar_file_name));
    }

    match ctx
        .narinfo_storage()
        .insert(&cache_infos, &path.narinfo_file_name, narinfo.to_string())
        .await
    {
        Ok(true) => Ok(narinfo_put::Response::Created),
        Ok(false) => Ok(narinfo_put::Response::AlreadyExists),
        Err(err) => Err(NarInfoPutRequestError::StoreNarInfo(err)),
    }
}

impl<NarStorageError: Into<StatusCode>, NarInfoStorageError>
    NarInfoPutRequestError<NarStorageError, NarInfoStorageError>
{
    fn into_status_code(self) -> StatusCode {
        match self {
            Self::InvalidNarInfoFileName(_)
            | Self::NarInfoNotUtf8(_)
            | Self::NarInfoStoreHashMismatch { .. }
            | Self::NarNotFound(_)
            | Self::ParseNarInfo(_) => StatusCode::BAD_REQUEST,
            Self::CheckNarExistence(err) => err.into(),
            Self::StoreNarInfo(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<NarStorageError, NarInfoStorageError> IntoResponse
    for NarInfoPutRequestError<NarStorageError, NarInfoStorageError>
where
    NarStorageError: fmt::Display + Into<StatusCode>,
    NarInfoStorageError: fmt::Display,
{
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        (self.into_status_code(), message).into_response()
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
