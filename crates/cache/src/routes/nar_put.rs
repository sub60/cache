//! TODO: docs.

use core::fmt;
use core::time::Duration;

use axum::body::Body;
use axum::extract::Path;
use axum::http::{self, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use nix_types::NarFileName;

use crate::context::ObjectStorageError;
use crate::object_gateway::{ObjectGateway, ObjectUrlRequest};
use crate::object_storage::ObjectStorageKey;
use crate::{Context, ObjectStorage};

pub(crate) const ROUTE: &str = "/nar/{nar_file_name}";

#[derive(serde::Deserialize)]
pub(super) struct NarPutPath {
    pub(super) nar_file_name: NarFileName,
}

/// The response returned to PUT requests to the NAR route.
#[derive(Debug)]
enum NarPutResponse {
    AlreadyExists,
    Redirect { nar_url: http::HeaderValue },
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

    let object_gateway = ctx.object_gateway();

    if object_gateway
        .storage()
        .contains_key(&nar_file_name)
        .await
        .map_err(NarPutRequestError::ObjectStorage)?
    {
        Ok(NarPutResponse::AlreadyExists)
    } else {
        let nar_url = object_gateway.object_url(ObjectUrlRequest {
            key: ObjectStorageKey::Nar(&nar_file_name),
            method: http::Method::PUT,
            min_ttl: Duration::from_mins(10),
        });
        Ok(NarPutResponse::Redirect {
            nar_url: http::HeaderValue::from_str(nar_url.as_ref())
                .expect("URL is valid header value"),
        })
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
            Self::Redirect { nar_url } => axum::response::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(header::LOCATION, nar_url)
                .body(Body::empty())
                .expect("valid response"),
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
