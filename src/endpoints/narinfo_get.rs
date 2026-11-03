//! TODO: docs.

use core::str::FromStr;
use core::{fmt, str};

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use compact_str::{CompactString, format_compact};
use nix_types::{NarFileName, NarInfo, NarInfoFromStrError};

use crate::api::endpoints::narinfo_get;
use crate::context::NarInfoStorageError;
use crate::{
    CacheControl as _,
    CacheInfos,
    Context,
    NarInfoStorage,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

/// The type of error that can occur when handling a GET request to the NARInfo
/// route.
#[derive(
    Debug, PartialEq, Eq, derive_more::Display, cauchy::Error, cauchy::From,
)]
enum NarInfoGetResponseError<NarInfoStorageError> {
    /// The NARInfo couldn't be retrieved from the NARInfo storage.
    GetNarInfo(NarInfoStorageError),

    /// The NARInfo bytes retrieved from the NARInfo storage were not valid
    /// UTF-8. This should basically never happen unless there's some kind of
    /// data corruption in the NARInfo storage or in the transport between the
    /// NARInfo storage and this service.
    NarInfoNotUtf8(str::Utf8Error),

    /// The NARInfo string retrieved from the NARInfo storage did not parse back
    /// into a valid [`NarInfo`]. Like the [`NarInfoNotUtf8`] error, this should
    /// also never happen unless the data is corrupted in some way.
    ParseNarInfo(#[from] NarInfoFromStrError<<EmptyString as FromStr>::Err>),
}

struct EmptyString;

impl endpoints::CacheEndpoint for narinfo_get::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PullFromCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |path, Extension(cache_infos)| {
                handler(path, cache_infos, &mut ctx).await
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: narinfo_get::Path,
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> Result<
    impl IntoResponse + use<Ctx>,
    NarInfoGetResponseError<NarInfoStorageError<Ctx>>,
> {
    let Some(narinfo_bytes) = ctx
        .narinfo_storage()
        .get(&cache_infos, &path.narinfo_file_name)
        .await
        .map_err(NarInfoGetResponseError::GetNarInfo)?
    else {
        return Ok((
            ctx.cache_control().no_store(), // The NARInfo can be pushed at any moment.
            narinfo_get::Response::NotFound,
        ));
    };

    let mut narinfo_without_url = str::from_utf8(narinfo_bytes.as_ref())
        .map_err(NarInfoGetResponseError::NarInfoNotUtf8)?
        .parse::<NarInfo<EmptyString>>()?;

    // NARInfos are stored unsigned, so we sign them on the fly with the cache's
    // current signing key.
    let cache_signature = cache_infos
        .signing_key
        .sign(narinfo_without_url.fingerprint().to_string().as_bytes());

    narinfo_without_url.signatures.push(cache_signature);

    let nar_filename = NarFileName::from(&narinfo_without_url);

    let nar_url = ctx.nar_storage().download_url(&cache_infos, &nar_filename);

    let cache_control =
        ctx.cache_control().narinfo(cache_infos.public_permissions);

    let response = narinfo_get::Response::Found(
        narinfo_without_url.with_url(nar_url.as_ref().to_owned()),
    );

    Ok((cache_control, response))
}

impl<NarInfoStorageError> NarInfoGetResponseError<NarInfoStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::GetNarInfo(_)
            | Self::NarInfoNotUtf8(_)
            | Self::ParseNarInfo(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<NarInfoStorageError: fmt::Display> IntoResponse
    for NarInfoGetResponseError<NarInfoStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl FromStr for EmptyString {
    type Err = CompactString;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self)
        } else {
            Err(format_compact!("expected empty string, found {s:?}"))
        }
    }
}
