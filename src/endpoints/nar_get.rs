//! TODO: docs.

use core::fmt;

use axum::Extension;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_get;
use crate::context::NarStorageError;
use crate::{
    CacheControl as _,
    CacheInfos,
    Context,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

const NAR_CONTENT_TYPE: HeaderValue =
    HeaderValue::from_static("application/x-nix-nar");

/// The type of error that can occur when handling a GET request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarGetRequestError<NarStorageError> {
    NarStorage(NarStorageError),
}

impl endpoints::CacheEndpoint for nar_get::Endpoint {
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
    path: nar_get::Path,
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> Result<
    impl IntoResponse + use<Ctx>,
    NarGetRequestError<NarStorageError<Ctx>>,
> {
    let Some(nar) = ctx
        .nar_storage()
        .get(&cache_infos.id, &path.nar_file_name)
        .await
        .map_err(NarGetRequestError::NarStorage)?
    else {
        return Ok((
            ctx.cache_control().no_store(), // The NAR can be pushed at any moment.
            nar_get::Response::NotFound.status_code().into_response(),
        ));
    };

    let mut response = nar.into_response();
    *response.status_mut() = nar_get::Response::Found.status_code();
    response.headers_mut().insert(header::CONTENT_TYPE, NAR_CONTENT_TYPE);

    let cache_control = ctx.cache_control().nar(cache_infos.public_permissions);

    Ok((cache_control, response))
}

impl<NarStorageError> NarGetRequestError<NarStorageError> {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NarStorage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<NarStorageError: fmt::Display> IntoResponse
    for NarGetRequestError<NarStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
