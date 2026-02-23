use axum::Router;

use crate::Context;
use crate::routes::{get_nar, get_narinfo, put_nar};

/// Returns the axum [`Router`] defining the cache server's API.
pub fn router(ctx: impl Context) -> Router {
    Router::new()
        .route(get_nar::ROUTE, get_nar::router(ctx.clone()))
        .route(get_nar::ROUTE_EXT, get_nar::router(ctx.clone()))
        .route(get_narinfo::ROUTE, get_narinfo::router(ctx.clone()))
        .route(put_nar::ROUTE, put_nar::router(ctx.clone()))
        .route(put_nar::ROUTE_EXT, put_nar::router(ctx.clone()))
}
