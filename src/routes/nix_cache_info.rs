//! TODO: docs.

use core::time::Duration;

use axum::Extension;
use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use smol_str::SmolStr;

use crate::cache_ctx::CacheCtx;
use crate::context::{self, Cdn, Context};
use crate::types::NixCacheInfo;

pub(crate) const ROUTE: &str = "/nix-cache-info";

/// The response returned to GET requests to the Nix cache info route.
struct NixCacheInfoResponse<Cdn: context::Cdn> {
    nix_cache_info: NixCacheInfo<SmolStr>,
    cache_headers: Cdn::CacheHeaders,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |cache_ctx| handler(cache_ctx, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Extension(cache_ctx): Extension<CacheCtx>,
    ctx: &mut Ctx,
) -> NixCacheInfoResponse<Ctx::Cdn> {
    NixCacheInfoResponse {
        nix_cache_info: NixCacheInfo {
            store_dir: cache_ctx.store_dir,
            want_mass_query: ctx.want_mass_query(),
            priority: cache_ctx.priority,
        },
        cache_headers: ctx.cdn().cache_headers(Duration::from_days(365)),
    }
}

impl<Cdn: context::Cdn> NixCacheInfoResponse<Cdn> {
    const CONTENT_TYPE: HeaderValue =
        HeaderValue::from_static("text/x-nix-cache-info");
}

impl<Cdn: context::Cdn> IntoResponse for NixCacheInfoResponse<Cdn> {
    fn into_response(self) -> axum::response::Response {
        let mut builder = axum::response::Response::builder()
            .header(header::CONTENT_TYPE, Self::CONTENT_TYPE);

        for (name, value) in self.cache_headers.into_iter() {
            builder = builder.header(name, value);
        }

        builder
            .body(Body::new(self.nix_cache_info.to_string()))
            .expect("headers and body are valid")
    }
}
