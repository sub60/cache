//! TODO: docs.

use axum::Extension;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use nix_types::NixCacheInfo;

use crate::api::endpoints::nix_cache_info;
use crate::{
    CacheControl as _,
    CacheInfos,
    Context,
    Permission,
    Permissions,
    endpoints,
};

impl endpoints::CacheEndpoint for nix_cache_info::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PullFromCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |Extension(cache_infos)| {
                handler(cache_infos, &mut ctx).await
            },
        )
    }
}

async fn handler<Ctx: Context>(
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> impl IntoResponse + use<Ctx> {
    let cache_control =
        ctx.cache_control().nix_cache_info(cache_infos.public_permissions);

    let response = nix_cache_info::Response(NixCacheInfo {
        store_dir: cache_infos.store_dir,
        want_mass_query: ctx.want_mass_query(),
        priority: cache_infos.priority,
    });

    (cache_control, response)
}
