//! The server's implementations of the API's endpoints.

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

use crate::api::{self, CachePath};
use crate::{Context, Permission, cache_access_layer};

/// An endpoint the server handles.
pub(crate) trait Endpoint: api::Endpoint {
    /// Returns the [`MethodRouter`] handling the endpoint's requests.
    fn method_router(ctx: impl Context) -> MethodRouter;
}

/// An endpoint acting on the cache its path identifies.
///
/// Its requests only reach its handler if they may access the cache, in which
/// case the handler can extract the cache's [`CacheInfos`](crate::CacheInfos)
/// as an [`Extension`](axum::Extension).
pub(crate) trait CacheEndpoint:
    api::Endpoint<Path: AsRef<CachePath> + Send>
{
    /// What requests must be allowed to do to the cache.
    const PERMISSION: Permission;

    /// Returns the [`MethodRouter`] handling the endpoint's requests that may
    /// access the cache.
    fn method_router(ctx: impl Context) -> MethodRouter;
}

/// Returns a [`MethodRouter`] routing the requests with the endpoint's method
/// to the handler.
fn method_router<E: api::Endpoint, H, T>(handler: H) -> MethodRouter
where
    H: Handler<T, ()>,
    T: 'static,
{
    let method_filter = MethodFilter::try_from(E::METHOD)
        .expect("axum supports the methods of all the endpoints");

    routing::on(method_filter, handler)
}

impl<E: CacheEndpoint> Endpoint for E {
    fn method_router(ctx: impl Context) -> MethodRouter {
        <E as CacheEndpoint>::method_router(ctx.clone())
            .route_layer(cache_access_layer::layer::<E>(ctx))
    }
}
