//! Sub60's production cache server running on Cloudflare Workers.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod cache_bucket;
mod cdn;

use core::convert::Infallible;

use cache::context::ComposedContext;
use cache_bucket::CacheBucket;
use cdn::CloudflareCdn;
use tower_service::Service;

const CLOUDFLARE_ACCOUNT_ID: &str = env!("CLOUDFLARE_ACCOUNT_ID");

/// The entry point for the Cloudflare Worker.
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> Result<axum::http::Response<axum::body::Body>, Infallible> {
    let context = ComposedContext::new()
        .with_cdn(CloudflareCdn)
        .with_object_gateway(CacheBucket::new(env));

    cache::router(context).call(req).await
}
