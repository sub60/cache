//! TODO: docs.

use core::fmt;
use core::time::Duration;

use axum::Extension;
use axum::body::Body;
use axum::http::{StatusCode, header};
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
            async move |path, Extension(cache_infos)| {
                handler(path, cache_infos, &mut ctx).await.map(into_response)
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: nar_put::Path,
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> Result<nar_put::Response, NarPutRequestError<NarStorageError<Ctx>>> {
    let nar_storage = ctx.nar_storage();

    if nar_storage
        .contains(&cache_infos.id, &path.nar_file_name)
        .await
        .map_err(NarPutRequestError::NarStorage)?
    {
        Ok(nar_put::Response::AlreadyExists)
    } else {
        let nar_url = nar_storage.upload_url(
            &cache_infos.id,
            &path.nar_file_name,
            Duration::from_mins(10),
        );
        Ok(nar_put::Response::Redirect(
            nar_url.as_ref().parse().expect("valid URL"),
        ))
    }
}

impl<NarStorageError> NarPutRequestError<NarStorageError> {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NarStorage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

fn into_response(response: nar_put::Response) -> axum::response::Response {
    let mut builder =
        axum::response::Response::builder().status(response.status_code());

    if let nar_put::Response::Redirect(nar_url) = &response {
        builder = builder.header(header::LOCATION, nar_url.to_string());
    }

    builder.body(Body::empty()).expect("valid response")
}

impl<NarStorageError: fmt::Display> IntoResponse
    for NarPutRequestError<NarStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
