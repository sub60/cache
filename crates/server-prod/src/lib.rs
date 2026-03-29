//! Sub60's production cache server running on Cloudflare Workers.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod cache_bucket;
mod cdn;
mod d1_cache_database;

use core::convert::Infallible;

use cache::context::ComposedContext;
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
        .with_cache_db(d1_cache_database::D1CacheDatabase::new(&env))
        .with_cdn(cdn::CloudflareCdn)
        .with_object_gateway(cache_bucket::CacheBucket::new(env));

    cache::router(context).call(req).await
}
