//! TODO: docs.

use core::fmt;

use axum::body::Body;
use axum::extract::Path;
use axum::http::{self, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::ObjectStorageError;
use crate::object_gateway::ObjectGateway;
use crate::types::NarFileName;
use crate::{Context, ObjectStorage};

pub(crate) const ROUTE: &str = "/nar/{nar_file_name}";

#[derive(serde::Deserialize)]
struct NarPutPath {
    nar_file_name: NarFileName,
}

/// The response returned to PUT requests to the NAR route.
#[derive(Debug)]
enum NarPutResponse {
    AlreadyExists,
    Redirect { nar_location: http::Uri },
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
    Path(NarPutPath { nar_file_name }): Path<NarPutPath>,
    ctx: &mut Ctx,
) -> Result<NarPutResponse, NarPutRequestError<ObjectStorageError<Ctx>>> {
    // TODO: validate that the user has write permissions for that cache.

    let object_storage = ctx.object_storage();

    let object_storage_key = (&nar_file_name).into();

    if object_storage
        .contains_key(&object_storage_key)
        .await
        .map_err(NarPutRequestError::ObjectStorage)?
    {
        Ok(NarPutResponse::AlreadyExists)
    } else {
        let nar_location = ctx.object_gateway().object_url(object_storage_key);
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
                    .header(
                        header::LOCATION,
                        header::HeaderValue::from_str(
                            &nar_location.to_string(),
                        )
                        .expect("URIs are valid header values"),
                    )
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
