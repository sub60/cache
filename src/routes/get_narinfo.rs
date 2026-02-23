//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::{self, Context};
use crate::types::{CacheName, NarInfo, NixStoreHash, UserName};

pub(crate) const ROUTE: &str = "/{user}/{cache}/{hash}.narinfo";

/// The response returned to GET requests to the NARInfo route.
pub struct GetNarInfoResponse<Cdn: context::Cdn> {
    nar_info: NarInfo,
    cache_headers: Cdn::NarInfoCacheHeaders,
}

/// The type of error that can occur when handling a GET request to the NARInfo
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum GetNarInfoResponseError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct GetNarInfoPath {
    user: UserName,
    cache: CacheName,
    hash: NixStoreHash,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<GetNarInfoPath>,
    _ctx: &mut Ctx,
) -> Result<GetNarInfoResponse<Ctx::Cdn>, GetNarInfoResponseError> {
    todo!()
}

impl<Cdn: context::Cdn> GetNarInfoResponse<Cdn> {
    const CONTENT_TYPE: HeaderValue =
        HeaderValue::from_static("text/x-nix-narinfo");
}

impl GetNarInfoResponseError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl<Cdn: context::Cdn> IntoResponse for GetNarInfoResponse<Cdn> {
    fn into_response(self) -> axum::response::Response {
        let mut builder = axum::response::Response::builder()
            .header(header::CONTENT_TYPE, Self::CONTENT_TYPE);

        for (name, value) in self.cache_headers.into_iter() {
            builder = builder.header(name, value);
        }

        builder
            .body(Body::new(self.nar_info.to_string()))
            .expect("headers and body are valid")
    }
}

impl IntoResponse for GetNarInfoResponseError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
