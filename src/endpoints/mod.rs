//! TODO: docs.

pub(crate) mod auth_github_actions;
pub(crate) mod nar_head;
pub(crate) mod nar_multiparts;
pub(crate) mod nar_multiparts_complete;
pub(crate) mod nar_multiparts_parts;
pub(crate) mod nar_put;
pub(crate) mod narinfo_get;
pub(crate) mod narinfo_put;
pub(crate) mod nix_cache_info;

use axum::handler::Handler;
use axum::routing::{self, MethodFilter, MethodRouter};

use crate::api::Endpoint;

/// An endpoint the server handles.
pub(crate) trait Serve: Endpoint {
    /// Returns the [`MethodRouter`] handling the endpoint's requests.
    fn method_router(ctx: impl crate::Context) -> MethodRouter;
}

/// Returns a [`MethodRouter`] routing the requests with the endpoint's method
/// to the handler.
fn method_router<E: Endpoint, H, T>(handler: H) -> MethodRouter
where
    H: Handler<T, ()>,
    T: 'static,
{
    let method_filter = MethodFilter::try_from(E::METHOD)
        .expect("axum supports the methods of all the endpoints");

    routing::on(method_filter, handler)
}
