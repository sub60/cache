//! TODO: docs.

use core::fmt;
use core::time::Duration;

use axum::body::Body;
use axum::extract::Path;
use axum::http::{self, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::routes::NAR_CONTENT_TYPE;
use crate::types::NarFileName;
use crate::{Context, ObjectStorage, PresignRequest};

pub(crate) const ROUTE: &str = crate::routes::nar_get::ROUTE;

/// The response returned to PUT requests to the NAR route.
#[derive(Debug)]
enum NarPutResponse {
    AlreadyExists,
    Redirect { nar_location: HeaderValue },
}

/// The type of error that can occur when handling a PUT request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarPutRequestError<ObjectStorageError> {
    ObjectStorage(ObjectStorageError),
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::put(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(nar_file_name): Path<NarFileName>,
    ctx: &mut Ctx,
) -> Result<
    NarPutResponse,
    NarPutRequestError<<Ctx::ObjectStorage as ObjectStorage>::Error>,
> {
    // TODO: validate that the user has write permissions for that cache.

    let object_storage_key = nar_file_name.into();

    let object_storage = ctx.object_storage();

    if object_storage
        .contains_key(&object_storage_key)
        .await
        .map_err(NarPutRequestError::ObjectStorage)?
    {
        Ok(NarPutResponse::AlreadyExists)
    } else {
        let request = PresignRequest {
            key: object_storage_key,
            method: http::Method::PUT,
            content_type: Some(NAR_CONTENT_TYPE),
            ttl: Some(Duration::from_mins(5)),
        };
        let nar_location = object_storage.presign(request).into();
        Ok(NarPutResponse::Redirect { nar_location })
    }
}

impl<ObjectStorageError> NarPutRequestError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::ObjectStorage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for NarPutResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::AlreadyExists => {
                (StatusCode::NO_CONTENT, Body::empty()).into_response()
            },
            Self::Redirect { nar_location } => {
                axum::response::Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(header::LOCATION, nar_location)
                    .body(Body::empty())
                    .expect("valid response")
            },
        }
    }
}

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarPutRequestError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
