//! TODO: docs.

use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{MethodRouter, get};

use crate::context::{self, Context};
use crate::types::{CacheName, NarInfo, NixStoreHash, UserName};

pub(crate) const ROUTE: &str = "/{user}/{cache}/{hash}.narinfo";

/// The type of error that can occur when handling a NarInfo request.
pub struct NarInfoResponse<Cdn: context::Cdn> {
    nar_info: NarInfo,
    cache_headers: Cdn::NarInfoCacheHeaders,
}

/// The type of error that can occur when handling a NarInfo request.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NarInfoResponseError {}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct NarInfoPath {
    user: UserName,
    cache: CacheName,
    hash: NixStoreHash,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    get(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(_path): Path<NarInfoPath>,
    _ctx: &mut Ctx,
) -> Result<NarInfoResponse<Ctx::Cdn>, NarInfoResponseError> {
    todo!()
}

impl<Cdn: context::Cdn> NarInfoResponse<Cdn> {
    const CONTENT_TYPE: HeaderValue =
        HeaderValue::from_static("text/x-nix-narinfo");
}

impl NarInfoResponseError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl<Cdn: context::Cdn> IntoResponse for NarInfoResponse<Cdn> {
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

impl IntoResponse for NarInfoResponseError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
