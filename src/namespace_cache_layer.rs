use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};

use crate::route_layer::RouteLayer;
use crate::{CacheDb, CacheName, Context, NamespaceName};

pub(crate) const ROUTE: &str = "/{namespace}/{cache}";

#[derive(serde::Deserialize)]
struct NamespaceCachePath {
    namespace: NamespaceName,
    cache: CacheName,
}

pub(crate) fn layer(ctx: impl Context) -> impl RouteLayer {
    middleware::from_fn(move |path, req, next| {
        let ctx = ctx.clone();
        async move { handler(path, req, next, &ctx).await }
    })
}

async fn handler<Ctx: Context>(
    Path(path): Path<NamespaceCachePath>,
    mut req: Request,
    next: Next,
    ctx: &Ctx,
) -> Response {
    let cache_infos =
        match ctx.cache_db().get_infos(path.namespace, path.cache).await {
            Ok(Some(infos)) => infos,
            Ok(None) => {
                return StatusCode::NOT_FOUND.into_response();
            },
            Err(db_err) => {
                let err_msg = db_err.to_string();
                return (db_err.into(), err_msg).into_response();
            },
        };

    req.extensions_mut().insert(cache_infos);
    next.run(req).await
}
