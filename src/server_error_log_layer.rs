//! A layer that logs every response with a 5xx status code.

use axum::body::{self, Body};
use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;

use crate::route_layer::RouteLayer;

/// The maximum size of a response body that gets included in the logs.
///
/// Server errors only carry a short error message, so this is just a safety
/// net against buffering an unexpectedly large body.
const MAX_LOGGED_BODY_SIZE: usize = 64 * 1024;

pub(crate) fn layer() -> impl RouteLayer {
    middleware::from_fn(handler)
}

async fn handler(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    if !response.status().is_server_error() {
        return response;
    }

    let (parts, body) = response.into_parts();

    let body = match body::to_bytes(body, MAX_LOGGED_BODY_SIZE).await {
        Ok(bytes) => {
            tracing::error!(
                %method,
                %uri,
                status = %parts.status,
                body = %String::from_utf8_lossy(&bytes),
                "server error",
            );
            Body::from(bytes)
        },
        Err(err) => {
            tracing::error!(
                %method,
                %uri,
                status = %parts.status,
                "server error, but couldn't read response body: {err}",
            );
            Body::empty()
        },
    };

    Response::from_parts(parts, body)
}
