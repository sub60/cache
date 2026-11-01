use axum::Router;

use crate::api::endpoints as api;
use crate::endpoints::Endpoint;
use crate::{Context, server_error_log_layer};

/// Returns the axum [`Router`] defining the cache server's API.
pub fn router(ctx: impl Context) -> Router {
    Router::new()
        .endpoint::<api::auth_github_actions::Endpoint>(ctx.clone())
        // Standard routes needed to support the Nix CLI client.
        .endpoint::<api::nar_head::Endpoint>(ctx.clone())
        .endpoint::<api::nar_put::Endpoint>(ctx.clone())
        .endpoint::<api::narinfo_get::Endpoint>(ctx.clone())
        .endpoint::<api::narinfo_put::Endpoint>(ctx.clone())
        .endpoint::<api::nix_cache_info::Endpoint>(ctx.clone())
        // Custom routes.
        .endpoint::<api::nar_multiparts::Endpoint>(ctx.clone())
        .endpoint::<api::nar_multiparts_parts::Endpoint>(ctx.clone())
        .endpoint::<api::nar_multiparts_complete::Endpoint>(ctx)
        // Log server errors from every route.
        .layer(server_error_log_layer::layer())
}

trait RouterExt {
    /// Routes the endpoint's requests to its handler.
    fn endpoint<E: Endpoint>(self, ctx: impl Context) -> Self;
}

impl RouterExt for Router {
    fn endpoint<E: Endpoint>(self, ctx: impl Context) -> Self {
        self.route(E::PATH, E::method_router(ctx))
    }
}
