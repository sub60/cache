use axum::Router;

use crate::routes::{
    auth_github_actions,
    nar_head,
    nar_multiparts,
    nar_multiparts_complete,
    nar_multiparts_parts,
    nar_put,
    narinfo_get,
    narinfo_put,
    nix_cache_info,
};
use crate::{Context, namespace_cache_layer, server_error_log_layer};

/// The prefix of the API routes. It can't be a namespace name, so it never
/// clashes with a cache's routes.
const API_ROUTE: &str = "/-/api/v1";

/// Returns the axum [`Router`] defining the cache server's API.
pub fn router(ctx: impl Context) -> Router {
    let cache_routes = Router::new()
        // Standard routes needed to support the Nix CLI client.
        .route(nar_head::ROUTE, nar_head::router(ctx.clone()))
        .route(nar_put::ROUTE, nar_put::router(ctx.clone()))
        .route(narinfo_get::ROUTE, narinfo_get::router(ctx.clone()))
        .route(narinfo_put::ROUTE, narinfo_put::router(ctx.clone()))
        .route(nix_cache_info::ROUTE, nix_cache_info::router(ctx.clone()))
        // Custom routes.
        .route(nar_multiparts::ROUTE, nar_multiparts::router(ctx.clone()))
        .route(
            nar_multiparts_parts::ROUTE,
            nar_multiparts_parts::router(ctx.clone()),
        )
        .route(
            nar_multiparts_complete::ROUTE,
            nar_multiparts_complete::router(ctx.clone()),
        );

    let api_routes = Router::new().route(
        auth_github_actions::ROUTE,
        auth_github_actions::router(ctx.clone()),
    );

    Router::new()
        .nest(API_ROUTE, api_routes)
        .nest(
            namespace_cache_layer::ROUTE,
            // Place all the cache routes under the /{namespace}/{cache} layer
            // which handles authentication/validation/etc.
            cache_routes.route_layer(namespace_cache_layer::layer(ctx)),
        )
        // Log server errors from every route.
        .layer(server_error_log_layer::layer())
}
