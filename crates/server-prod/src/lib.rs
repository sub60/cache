//! Sub60's production cache server running on Cloudflare Workers.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod cdn;
mod object_storage;

use core::convert::Infallible;

use cache::context::ComposedContext;
use cdn::CloudflareCdn;
use object_storage::R2ObjectStorage;
use tower_service::Service;

/// The entry point for the Cloudflare Worker.
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    _env: worker::Env,
    _ctx: worker::Context,
) -> Result<axum::http::Response<axum::body::Body>, Infallible> {
    let context = ComposedContext::new()
        .with_cdn(CloudflareCdn)
        .with_object_gateway(R2ObjectStorage);

    cache::router(context).call(req).await
}
