use axum::Router;

use crate::{Context, routes};

/// Returns the axum [`Router`] defining the cache server's API.
pub fn router(ctx: impl Context) -> Router {
    Router::new()
        .route(routes::narinfo::ROUTE, routes::narinfo::router(ctx.clone()))
}
