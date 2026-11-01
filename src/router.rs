use axum::Router;

use crate::api::{Endpoint as _, endpoints as api};
use crate::endpoints::{
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

/// Returns the axum [`Router`] defining the cache server's API.
pub fn router(ctx: impl Context) -> Router {
    let cache_routes = Router::new()
        // Standard routes needed to support the Nix CLI client.
        .route(api::nar_head::Endpoint::PATH, nar_head::router(ctx.clone()))
        .route(api::nar_put::Endpoint::PATH, nar_put::router(ctx.clone()))
        .route(
            api::narinfo_get::Endpoint::PATH,
            narinfo_get::router(ctx.clone()),
        )
        .route(
            api::narinfo_put::Endpoint::PATH,
            narinfo_put::router(ctx.clone()),
        )
        .route(
            api::nix_cache_info::Endpoint::PATH,
            nix_cache_info::router(ctx.clone()),
        )
        // Custom routes.
        .route(
            api::nar_multiparts::Endpoint::PATH,
            nar_multiparts::router(ctx.clone()),
        )
        .route(
            api::nar_multiparts_parts::Endpoint::PATH,
            nar_multiparts_parts::router(ctx.clone()),
        )
        .route(
            api::nar_multiparts_complete::Endpoint::PATH,
            nar_multiparts_complete::router(ctx.clone()),
        )
        // Place all the cache routes behind the /{namespace}/{cache} layer
        // which handles authentication/validation/etc.
        .route_layer(namespace_cache_layer::layer(ctx.clone()));

    Router::new()
        .route(
            api::auth_github_actions::Endpoint::PATH,
            auth_github_actions::router(ctx.clone()),
        )
        .merge(cache_routes)
        // Log server errors from every route.
        .layer(server_error_log_layer::layer())
}
