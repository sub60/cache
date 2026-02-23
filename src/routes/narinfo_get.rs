//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::{self, Context};
use crate::types::{NarInfo, NixStoreHash};

pub(crate) const ROUTE: &str = "/{store_hash}.narinfo";

/// The response returned to GET requests to the NARInfo route.
struct NarInfoGetResponse<Cdn: context::Cdn> {
    nar_info: NarInfo,
    cache_headers: Cdn::CacheHeaders,
}

/// The type of error that can occur when handling a GET request to the NARInfo
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarInfoGetResponseError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct NarInfoGetPath {
    store_hash: NixStoreHash,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<NarInfoGetPath>,
    _ctx: &mut Ctx,
) -> Result<NarInfoGetResponse<Ctx::Cdn>, NarInfoGetResponseError> {
    todo!()
}

impl<Cdn: context::Cdn> NarInfoGetResponse<Cdn> {
    const CONTENT_TYPE: HeaderValue =
        HeaderValue::from_static("text/x-nix-narinfo");
}

impl NarInfoGetResponseError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl<Cdn: context::Cdn> IntoResponse for NarInfoGetResponse<Cdn> {
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

impl IntoResponse for NarInfoGetResponseError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
