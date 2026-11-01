use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};

use crate::api::CachePath;
use crate::route_layer::RouteLayer;
use crate::{CacheDb, Context};

pub(crate) fn layer(ctx: impl Context) -> impl RouteLayer {
    middleware::from_fn(move |path, req, next| {
        let ctx = ctx.clone();
        async move { handler(path, req, next, &ctx).await }
    })
}

async fn handler<Ctx: Context>(
    path: CachePath,
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
