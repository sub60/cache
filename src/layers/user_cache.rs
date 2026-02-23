use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use either::Either;

use crate::Context;
use crate::cache_ctx::CacheCtx;
use crate::layers::RouteLayer;
use crate::types::{CacheName, NixStoreLiteral, UserName};
use crate::user_ctx::UserCtx;

pub(crate) const ROUTE: &str = "/{user}/{cache}";

#[derive(serde::Deserialize)]
struct UserCachePath {
    user: UserName,
    cache: CacheName,
}

/// The type of error that can occur when handling a PUT request to the NARInfo
/// route.
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum UserCacheError {}

pub(crate) fn layer(ctx: impl Context) -> impl RouteLayer {
    middleware::from_fn::<_, (Path<UserCachePath>, Request)>(
        move |Path(path), mut req: Request, next: Next| {
            let mut ctx = ctx.clone();
            async move {
                let (user_ctx, cache_ctx) = handler(path, &mut ctx).await?;
                let extensions = req.extensions_mut();
                extensions.insert(user_ctx);
                extensions.insert(cache_ctx);
                Ok::<_, UserCacheError>(next.run(req).await)
            }
        },
    )
}

async fn handler<Ctx: Context>(
    path: UserCachePath,
    _ctx: &mut Ctx,
) -> Result<(UserCtx, CacheCtx), UserCacheError> {
    let user_ctx = UserCtx { name: path.user };

    let cache_priority = 40;

    let cache_store_dir = Either::Left(NixStoreLiteral);

    let cache_ctx = CacheCtx {
        name: path.cache,
        priority: cache_priority,
        store_dir: cache_store_dir,
    };

    Ok((user_ctx, cache_ctx))
}

impl UserCacheError {
    fn status_code(&self) -> StatusCode {
        match *self {}
    }
}

impl IntoResponse for UserCacheError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
