//! TODO: docs.

use core::time::Duration;

use axum::Extension;
use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use nix_types::NixCacheInfo;
use smol_str::SmolStr;

use crate::{CacheDb, CacheInfos, Cdn as _, Context};

pub(crate) const ROUTE: &str = "/nix-cache-info";

/// The response returned to GET requests to the Nix cache info route.
struct NixCacheInfoResponse<Cdn: crate::Cdn> {
    nix_cache_info: NixCacheInfo<SmolStr>,
    cache_headers: Cdn::CacheHeaders,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::get(async move |Extension(cache_infos)| {
        handler(cache_infos, &mut ctx).await
    })
}

async fn handler<Ctx: Context>(
    cache_infos: CacheInfos<<Ctx::CacheDb as CacheDb>::CacheId>,
    ctx: &mut Ctx,
) -> NixCacheInfoResponse<Ctx::Cdn> {
    NixCacheInfoResponse {
        nix_cache_info: NixCacheInfo {
            store_dir: cache_infos.store_dir,
            want_mass_query: ctx.want_mass_query(),
            priority: cache_infos.priority,
        },
        cache_headers: ctx.cdn().cache_headers(Duration::from_days(365)),
    }
}

impl<Cdn: crate::Cdn> NixCacheInfoResponse<Cdn> {
    const CONTENT_TYPE: HeaderValue =
        HeaderValue::from_static("text/x-nix-cache-info");
}

impl<Cdn: crate::Cdn> IntoResponse for NixCacheInfoResponse<Cdn> {
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
