//! Sub60's production cache server running on Cloudflare Workers.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod cdn;
mod object_storage;

use core::convert::Infallible;

use cache::context::ComposedContext;
use cdn::CloudflareCdn;
use object_storage::{R2ObjectStorage, R2S3Credentials};
use tower_service::Service;

/// The entry point for the Cloudflare Worker.
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> Result<axum::http::Response<axum::body::Body>, Infallible> {
    let bucket = env
        .bucket(env!("R2_BUCKET_BINDING_NAME"))
        .expect("R2 bucket binding not found");

    let s3_creds = R2S3Credentials {
        account_id: env
            .secret("CLOUDFLARE_ACCOUNT_ID")
            .expect("CLOUDFLARE_ACCOUNT_ID secret not found")
            .to_string(),
        access_key_id: env
            .secret("R2_ACCESS_KEY_ID")
            .expect("R2_ACCESS_KEY_ID secret not found")
            .to_string(),
        secret_access_key: env
            .secret("R2_SECRET_ACCESS_KEY")
            .expect("R2_SECRET_ACCESS_KEY secret not found")
            .to_string(),
        bucket_name: env
            .var("R2_BUCKET_NAME")
            .expect("R2_BUCKET_NAME var not found")
            .to_string(),
    };

    let context = ComposedContext::new()
        .with_cdn(CloudflareCdn)
        .with_object_gateway(R2ObjectStorage::new(bucket, s3_creds));

    cache::router(context).call(req).await
}
