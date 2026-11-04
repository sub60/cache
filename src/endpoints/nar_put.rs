//! TODO: docs.

use core::fmt;

use axum::Extension;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_put;
use crate::context::NarStorageError;
use crate::{
    CacheInfos,
    Context,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

/// The type of error that can occur when handling a PUT request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarPutRequestError<NarStorageError> {
    NarStorage(NarStorageError),
}

impl endpoints::CacheEndpoint for nar_put::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PushToCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |path, Extension(cache_infos), request| {
                handler(path, cache_infos, request, &mut ctx)
                    .await
                    .map(|response| response.status_code())
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: nar_put::Path,
    cache_infos: CacheInfos,
    request: Request,
    ctx: &mut Ctx,
) -> Result<nar_put::Response, NarPutRequestError<NarStorageError<Ctx>>> {
    match ctx
        .nar_storage()
        .insert(&cache_infos, &path.nar_file_name, request)
        .await
    {
        Ok(true) => Ok(nar_put::Response::Created),
        Ok(false) => Ok(nar_put::Response::AlreadyExists),
        Err(err) => Err(NarPutRequestError::NarStorage(err)),
    }
}

impl<NarStorageError: Into<StatusCode>> NarPutRequestError<NarStorageError> {
    fn into_status_code(self) -> StatusCode {
        match self {
            Self::NarStorage(err) => err.into(),
        }
    }
}

impl<NarStorageError> IntoResponse for NarPutRequestError<NarStorageError>
where
    NarStorageError: fmt::Display + Into<StatusCode>,
{
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        (self.into_status_code(), message).into_response()
    }
}
