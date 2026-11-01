//! TODO: docs.

use core::fmt;
use core::time::Duration;

use axum::body::Body;
use axum::http::{self, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_put;
use crate::context::ObjectStorageError;
use crate::object_gateway::{ObjectGateway, ObjectUrlRequest};
use crate::object_storage::ObjectStorageKey;
use crate::{Context, ObjectStorage, endpoints};

/// The type of error that can occur when handling a PUT request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarPutRequestError<ObjectStorageError> {
    ObjectStorage(ObjectStorageError),
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    endpoints::method_router::<nar_put::Endpoint, _, _>(async move |path| {
        handler(path, &mut ctx).await.map(into_response)
    })
}

async fn handler<Ctx: Context>(
    path: nar_put::Path,
    ctx: &mut Ctx,
) -> Result<nar_put::Response, NarPutRequestError<ObjectStorageError<Ctx>>> {
    // TODO: validate that the user has write permissions for that cache.

    let object_gateway = ctx.object_gateway();

    if object_gateway
        .storage()
        .contains_key(ObjectStorageKey::Nar(&path.nar_file_name))
        .await
        .map_err(NarPutRequestError::ObjectStorage)?
    {
        Ok(nar_put::Response::AlreadyExists)
    } else {
        let nar_url = object_gateway.object_url(ObjectUrlRequest {
            key: ObjectStorageKey::Nar(&path.nar_file_name),
            method: http::Method::PUT,
            min_ttl: Duration::from_mins(10),
        });
        Ok(nar_put::Response::Redirect(
            nar_url.as_ref().parse().expect("valid URL"),
        ))
    }
}

impl<ObjectStorageError> NarPutRequestError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::ObjectStorage(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarPutRequestError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
