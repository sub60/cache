//! TODO: docs.

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::narinfo_head;
use crate::{
    CacheControl as _,
    CacheInfos,
    Context,
    NarInfoStorage,
    Permission,
    Permissions,
    endpoints,
};

impl endpoints::CacheEndpoint for narinfo_head::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PullFromCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |path, Extension(cache_infos)| {
                handler(path, cache_infos, &mut ctx).await
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: narinfo_head::Path,
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> Result<impl IntoResponse + use<Ctx>, impl IntoResponse + use<Ctx>> {
    ctx.narinfo_storage()
        .contains(&cache_infos, &path.narinfo_file_name)
        .await
        .map(|exists| {
            let response = if exists {
                narinfo_head::Response::Exists
            } else {
                narinfo_head::Response::NotFound
            };
            // Clients check whether NARInfos exist right before pushing them.
            (ctx.cache_control().no_store(), response.status_code())
        })
        .map_err(|storage_err| {
            (StatusCode::INTERNAL_SERVER_ERROR, storage_err.to_string())
        })
}
