//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::Context;
use crate::types::NarFileName;

pub(crate) const ROUTE: &str = "/nar/{nar_file_name}";

/// The response returned to GET requests to the NAR route.
#[derive(Debug)]
struct NarGetResponse {
    nar_location: HeaderValue,
}

/// The type of error that can occur when handling a GET request to the NAR
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarGetRequestError {}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_nar_file_name): Path<NarFileName>,
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
