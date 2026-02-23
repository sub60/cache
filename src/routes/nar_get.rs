//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::Context;
use crate::types::{CompressionExtension, Nix32HashStr};

pub(crate) const ROUTE: &str = "/nar/{file_hash}.nar";
pub(crate) const ROUTE_EXT: &str = "/nar/{file_hash}.nar.{ext}";

/// The response returned to GET requests to the NAR route.
#[derive(Debug)]
struct NarGetResponse {
    nar_location: HeaderValue,
}

/// The type of error that can occur when handling a GET request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarGetRequestError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct NarGetPath {
    file_hash: Nix32HashStr<32>,
    ext: Option<CompressionExtension>,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<NarGetPath>,
    _ctx: &mut Ctx,
) -> Result<NarGetResponse, NarGetRequestError> {
    todo!()
}

impl NarGetRequestError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl IntoResponse for NarGetResponse {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(header::LOCATION, self.nar_location)
            .body(Body::empty())
            .expect("valid response")
    }
}

impl IntoResponse for NarGetRequestError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
