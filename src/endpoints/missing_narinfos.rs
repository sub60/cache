//! TODO: docs.

use core::fmt;

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::missing_narinfos;
use crate::context::NarInfoStorageError;
use crate::from_str_body::FromStrBody;
use crate::{
    CacheInfos,
    Context,
    NarInfoStorage,
    Permission,
    Permissions,
    endpoints,
};

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum MissingNarInfosError<NarInfoStorageError> {
    #[display("couldn't look up the NARInfos: {_0}")]
    LookUpNarInfos(NarInfoStorageError),
    #[display("request has {_0} store hashes (maximum {_1})")]
    TooManyStoreHashes(usize, usize),
}

impl endpoints::CacheEndpoint for missing_narinfos::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PullFromCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |Extension(cache_infos), body| {
                handler(cache_infos, body, &mut ctx).await
            },
        )
    }
}

async fn handler<Ctx: Context>(
    cache_infos: CacheInfos,
    FromStrBody(store_hashes): FromStrBody<missing_narinfos::RequestBody>,
    ctx: &mut Ctx,
) -> Result<
    missing_narinfos::ResponseBody,
    MissingNarInfosError<NarInfoStorageError<Ctx>>,
> {
    if store_hashes.len() > missing_narinfos::MAX_STORE_HASHES {
        return Err(MissingNarInfosError::TooManyStoreHashes(
            store_hashes.len(),
            missing_narinfos::MAX_STORE_HASHES,
        ));
    }

    ctx.narinfo_storage()
        .missing(&cache_infos, &store_hashes)
        .await
        .map(missing_narinfos::StoreHashes)
        .map_err(MissingNarInfosError::LookUpNarInfos)
}

impl<NarInfoStorageError> MissingNarInfosError<NarInfoStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::LookUpNarInfos(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TooManyStoreHashes(..) => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}

impl<NarInfoStorageError: fmt::Display> IntoResponse
    for MissingNarInfosError<NarInfoStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
