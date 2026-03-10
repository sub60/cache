use axum::extract::{Path, Request};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use smol_str::SmolStr;
use types::{CacheName, NixStoreLiteral, UserName};

use crate::Context;
use crate::cache_ctx::{CacheCtx, CacheVisibility};
use crate::layers::RouteLayer;
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
    middleware::from_fn(
        move |Path(path): Path<UserCachePath>, mut req: Request, next: Next| {
            let mut ctx = ctx.clone();
            async move {
                let (user_ctx, cache_ctx) =
                    handler(path.user, path.cache, &mut ctx).await?;
                let extensions = req.extensions_mut();
                extensions.insert(user_ctx);
                extensions.insert(cache_ctx);
                Ok::<_, UserCacheError>(next.run(req).await)
            }
        },
    )
}

async fn handler<Ctx: Context>(
    user_name: UserName,
    cache_name: CacheName,
    _ctx: &mut Ctx,
) -> Result<(UserCtx, CacheCtx), UserCacheError> {
    let user_ctx = UserCtx { name: user_name };

    let cache_priority = 40;

    let cache_store_dir = SmolStr::new_static(NixStoreLiteral::STR);

    let cache_visibility = CacheVisibility::Public;

    let cache_ctx = CacheCtx {
        name: cache_name,
        priority: cache_priority,
        store_dir: cache_store_dir,
        visibility: cache_visibility,
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
