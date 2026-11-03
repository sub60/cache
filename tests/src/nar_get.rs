use axum::body::{self, Body};
use axum::http::{Request, Response, StatusCode, header};
use nix_types::NarFileName;
use sub60_cache::Context;
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn stored_nar_is_served() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    test_ctx.nar_storage().insert(&nar_file_name, nar_bytes.clone());

    let mut app = test_ctx.app_router();

    let response = get_nar(&mut app, "namespace/cache", &nar_file_name).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/x-nix-nar"
    );
    let body = body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, nar_bytes);
}

#[tokio::test]
async fn missing_nar_is_not_found() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    let mut app = test_ctx.app_router();

    let response = get_nar(&mut app, "namespace/cache", &nar_file_name).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn get_nar(
    app: &mut axum::Router,
    cache: &str,
    nar_file_name: &NarFileName,
) -> Response<Body> {
    let request = Request::builder()
        .uri(format!("/{cache}/nar/{nar_file_name}"))
        .body(Body::empty())
        .expect("valid request");

    let Ok(response) = app.call(request).await;

    response
}
