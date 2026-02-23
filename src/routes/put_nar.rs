//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::Context;
use crate::types::{CacheName, CompressionExtension, Nix32HashStr, UserName};

pub(crate) const ROUTE: &str = "/{user}/{cache}/{file_hash}.nar";
pub(crate) const ROUTE_EXT: &str = "/{user}/{cache}/{file_hash}.nar.{ext}";

/// The response returned to PUT requests to the NAR route.
#[derive(Debug)]
pub struct PutNarResponse {
    nar_location: HeaderValue,
}

/// The type of error that can occur when handling a PUT request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum PutNarRequestError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct PutNarPath {
    user: UserName,
    cache: CacheName,
    file_hash: Nix32HashStr<32>,
    ext: Option<CompressionExtension>,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::put(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<PutNarPath>,
    _ctx: &mut Ctx,
) -> Result<PutNarResponse, PutNarRequestError> {
    todo!()
}

impl PutNarRequestError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl IntoResponse for PutNarResponse {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(header::LOCATION, self.nar_location)
            .body(Body::empty())
            .expect("valid response")
    }
}

impl IntoResponse for PutNarRequestError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
