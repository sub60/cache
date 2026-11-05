use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use nix_types::StoreHash;
use sub60_cache::api::endpoints::missing_narinfos::{self, StoreHashes};
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn missing_narinfos_are_listed_in_request_order() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "foo");
    test_ctx.db_mut().add_cache("namespace", "bar");

    let nix = MockNixClient::default();

    let pushed = nix.store().add_file("pushed", b"pushed").await;
    let first = nix.store().add_file("first", b"first").await;
    let second = nix.store().add_file("second", b"second").await;

    nix.copy([pushed.clone()])
        .to(test_ctx.app_router(), "namespace", "foo")
        .token(test_ctx.write_token("namespace", "foo"))
        .await
        .unwrap();

    let mut app = test_ctx.app_router();

    let request = [*first.hash(), *pushed.hash(), *second.hash()];

    let (status, missing) =
        missing_narinfos(&mut app, "namespace/foo", &request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(missing, StoreHashes(vec![*first.hash(), *second.hash()]));

    // The NARInfo is only in the cache it was pushed to.
    let (status, missing) =
        missing_narinfos(&mut app, "namespace/bar", &request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(missing, StoreHashes(request.to_vec()));
}

#[tokio::test]
async fn missing_narinfos_refuses_too_many_store_hashes() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "foo");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    let request =
        vec![*store_path.hash(); missing_narinfos::MAX_STORE_HASHES + 1];

    let (status, _) =
        missing_narinfos(&mut test_ctx.app_router(), "namespace/foo", &request)
            .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

async fn missing_narinfos(
    app: &mut axum::Router,
    base: &str,
    store_hashes: &[StoreHash],
) -> (StatusCode, StoreHashes) {
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/{base}/missing-narinfos"))
        .body(Body::from(StoreHashes(store_hashes.to_vec()).to_string()))
        .expect("valid request");

    let Ok(response) = app.call(request).await;

    let status = response.status();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read missing-narinfos body");

    let missing = if status == StatusCode::OK {
        str::from_utf8(&body)
            .expect("UTF-8 body")
            .parse()
            .expect("valid store hashes")
    } else {
        StoreHashes::default()
    };

    (status, missing)
}
