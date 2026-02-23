use axum::Router;

use crate::Context;
use crate::layers::user_cache;
use crate::routes::{
    nar_get,
    nar_put,
    narinfo_get,
    narinfo_put,
    nix_cache_info,
};

/// Returns the axum [`Router`] defining the cache server's API.
pub fn router(ctx: impl Context) -> Router {
    let cache_routes = Router::new()
        .route(nar_get::ROUTE, nar_get::router(ctx.clone()))
        .route(nar_get::ROUTE_EXT, nar_get::router(ctx.clone()))
        .route(nar_put::ROUTE, nar_put::router(ctx.clone()))
        .route(nar_put::ROUTE_EXT, nar_put::router(ctx.clone()))
        .route(narinfo_get::ROUTE, narinfo_get::router(ctx.clone()))
        .route(narinfo_put::ROUTE, narinfo_put::router(ctx.clone()))
        .route(nix_cache_info::ROUTE, nix_cache_info::router(ctx.clone()));

    Router::new()
        // Place all the cache routes behind the /{user}/{cache} layer which
        // handles authentication/validation/etc.
        .nest(
            user_cache::ROUTE,
            cache_routes.route_layer(user_cache::layer(ctx)),
        )
}
