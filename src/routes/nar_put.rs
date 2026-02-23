//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::Context;
use crate::types::{CompressionExtension, Nix32HashStr};

pub(crate) const ROUTE: &str = crate::routes::nar_get::ROUTE;
pub(crate) const ROUTE_EXT: &str = crate::routes::nar_get::ROUTE_EXT;

/// The response returned to PUT requests to the NAR route.
#[derive(Debug)]
struct NarPutResponse {
    nar_location: HeaderValue,
}

/// The type of error that can occur when handling a PUT request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarPutRequestError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct PutNarPath {
    file_hash: Nix32HashStr<32>,
    ext: Option<CompressionExtension>,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::put(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<PutNarPath>,
    _ctx: &mut Ctx,
) -> Result<NarPutResponse, NarPutRequestError> {
    todo!()
}

impl NarPutRequestError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl IntoResponse for NarPutResponse {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(header::LOCATION, self.nar_location)
            .body(Body::empty())
            .expect("valid response")
    }
}

impl IntoResponse for NarPutRequestError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
