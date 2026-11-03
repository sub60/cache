//! TODO: docs.

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_head;
use crate::{
    CacheInfos,
    Context,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

impl endpoints::CacheEndpoint for nar_head::Endpoint {
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
    path: nar_head::Path,
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> Result<StatusCode, impl IntoResponse + use<Ctx>> {
    ctx.nar_storage()
        .contains(&cache_infos.id, &path.nar_file_name)
        .await
        .map(|exists| {
            let response = if exists {
                nar_head::Response::Exists
            } else {
                nar_head::Response::NotFound
            };
            response.status_code()
        })
        .map_err(|storage_err| {
            (StatusCode::INTERNAL_SERVER_ERROR, storage_err.to_string())
        })
}
