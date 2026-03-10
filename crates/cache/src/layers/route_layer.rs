use core::convert::Infallible;

use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::Route;
use tower_layer::Layer;
use tower_service::Service;

pub(crate) trait RouteLayer:
    Layer<Route, Service: RouteLayerService> + Clone + Send + Sync + 'static
{
}

pub(crate) trait RouteLayerService:
    Service<
        Request,
        Response: IntoResponse + 'static,
        Error: Into<Infallible> + 'static,
        Future: Send + 'static,
    > + Clone
    + Send
    + Sync
    + 'static
{
}

impl<T> RouteLayer for T
where
    T: Layer<Route> + Clone + Send + Sync + 'static,
    T::Service: RouteLayerService,
{
}

impl<T> RouteLayerService for T
where
    T: Service<Request> + Clone + Send + Sync + 'static,
    T::Response: IntoResponse + 'static,
    T::Error: Into<Infallible> + 'static,
    T::Future: Send + 'static,
{
}
