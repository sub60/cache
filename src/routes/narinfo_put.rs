//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::Context;
use crate::types::{CacheName, NixStoreHash, UserName};

pub(crate) const ROUTE: &str = crate::routes::narinfo_get::ROUTE;

/// The response returned to PUT requests to the NARInfo route.
#[derive(Debug, Copy, Clone)]
#[expect(dead_code)]
pub enum NarInfoPutResponse {
    /// The NARInfo already existed in the user's cache, so the request was a
    /// no-op.
    AlreadyExisted,
    /// The NARInfo was successfully created.
    Created,
}

/// The type of error that can occur when handling a PUT request to the NARInfo
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NarInfoPutRequestError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct NarInfoPutPath {
    user: UserName,
    cache: CacheName,
    store_hash: NixStoreHash,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::put(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<NarInfoPutPath>,
    _ctx: &mut Ctx,
) -> Result<NarInfoPutResponse, NarInfoPutRequestError> {
    todo!()
}

impl NarInfoPutResponse {
    fn status_code(self) -> StatusCode {
        match self {
            Self::AlreadyExisted => StatusCode::NO_CONTENT,
            Self::Created => StatusCode::CREATED,
        }
    }
}

impl NarInfoPutRequestError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl IntoResponse for NarInfoPutResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), Body::empty()).into_response()
    }
}

impl IntoResponse for NarInfoPutRequestError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
