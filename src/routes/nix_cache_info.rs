//! TODO: docs.

use core::time::Duration;

use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::{self, Cdn, Context};
use crate::types::{NixCacheInfo, NixStoreLiteral};

pub(crate) const ROUTE: &str = "/nix-cache-info";

/// The response returned to GET requests to the Nix cache info route.
struct NixCacheInfoResponse<Cdn: context::Cdn> {
    nix_cache_info: NixCacheInfo,
    cache_headers: Cdn::CacheHeaders,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |()| handler(&mut ctx).await)
}

async fn handler<Ctx: Context>(
    ctx: &mut Ctx,
) -> NixCacheInfoResponse<Ctx::Cdn> {
    NixCacheInfoResponse {
        nix_cache_info: NixCacheInfo {
            store_dir: NixStoreLiteral,
            want_mass_query: ctx.want_mass_query(),
            priority: 40,
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
