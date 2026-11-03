use axum::body::Body;
use axum::http::{Request, Response, header};
use nix_types::{NarFileName, StorePath};
use sub60_cache::Permissions;
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn shared_caches_can_store_what_public_caches_have() {
    let (test_ctx, store_path, nar_file_name) = pushed_store_path().await;

    let narinfo = get(&test_ctx, &narinfo_path(&store_path), None).await;
    assert_eq!(cache_control(&narinfo), "public, max-age=86400");

    let nar = get(&test_ctx, &nar_path(&nar_file_name), None).await;
    assert_eq!(cache_control(&nar), "public, max-age=31536000, immutable");

    let nix_cache_info = get(&test_ctx, NIX_CACHE_INFO_PATH, None).await;
    assert_eq!(cache_control(&nix_cache_info), "public, max-age=3600");
}

#[tokio::test]
async fn shared_caches_cant_store_what_caches_dont_have() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;

    let narinfo_response =
        get(&test_ctx, &narinfo_path(&store_path), None).await;
    assert_eq!(cache_control(&narinfo_response), "no-store");

    let nar_response =
        get(&test_ctx, &nar_path(&NarFileName::from(&narinfo)), None).await;
    assert_eq!(cache_control(&nar_response), "no-store");
}

#[tokio::test]
async fn shared_caches_cant_store_what_private_caches_have() {
    let (mut test_ctx, store_path, nar_file_name) = pushed_store_path().await;
    test_ctx.db_mut().set_public_permissions(
        "namespace",
        "cache",
        Permissions::new([]),
    );
    let token = test_ctx.write_token("namespace", "cache");
    let authorization = format!("Bearer {token}");

    for path in [
        narinfo_path(&store_path),
        nar_path(&nar_file_name),
        NIX_CACHE_INFO_PATH.to_owned(),
    ] {
        let response = get(&test_ctx, &path, Some(&authorization)).await;
        assert_eq!(cache_control(&response), "private", "{path}");
    }
}

const NIX_CACHE_INFO_PATH: &str = "/namespace/cache/nix-cache-info";

/// Returns a context with a public `namespace/cache` cache, along with the
/// store path pushed to it and the file name of its NAR.
async fn pushed_store_path() -> (TestContext, StorePath, NarFileName) {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;

    nix.copy([store_path.clone()])
        .to(test_ctx.app_router(), "namespace", "cache")
        .token(test_ctx.write_token("namespace", "cache"))
        .await
        .expect("store path is pushed");

    (test_ctx, store_path, NarFileName::from(&narinfo))
}

fn narinfo_path(store_path: &StorePath) -> String {
    format!("/namespace/cache/{}.narinfo", store_path.hash())
}

fn nar_path(nar_file_name: &NarFileName) -> String {
    format!("/namespace/cache/nar/{nar_file_name}")
}

fn cache_control(response: &Response<Body>) -> &str {
    response.headers()[header::CACHE_CONTROL]
        .to_str()
        .expect("Cache-Control is ASCII")
}

async fn get(
    test_ctx: &TestContext,
    path: &str,
    authorization: Option<&str>,
) -> Response<Body> {
    let mut request = Request::builder().uri(path);

    if let Some(authorization) = authorization {
        request = request.header(header::AUTHORIZATION, authorization);
    }

    let request = request.body(Body::empty()).expect("valid request");

    let Ok(response) = test_ctx.app_router().call(request).await;

    response
}
