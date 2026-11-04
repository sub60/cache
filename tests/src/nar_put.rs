use axum::body::{Body, Bytes};
use axum::http::{Method, Request, Response, StatusCode, header};
use nix_types::NarFileName;
use sub60_cache::Context;
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn nar_is_stored() {
    let (mut test_ctx, nar_file_name, nar_bytes) = test_ctx_and_nar().await;

    let response = put_nar(&mut test_ctx, &nar_file_name, nar_bytes).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(get_nar(&test_ctx, &nar_file_name).await, StatusCode::OK);
}

#[tokio::test]
async fn stored_nar_is_left_as_is() {
    let (mut test_ctx, nar_file_name, nar_bytes) = test_ctx_and_nar().await;

    test_ctx.nar_storage().insert_unchecked(&nar_file_name, nar_bytes.clone());

    let response = put_nar(&mut test_ctx, &nar_file_name, nar_bytes).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn nar_with_wrong_hash_is_rejected() {
    let (mut test_ctx, nar_file_name, _) = test_ctx_and_nar().await;

    let response =
        put_nar(&mut test_ctx, &nar_file_name, Bytes::from("not it")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(get_nar(&test_ctx, &nar_file_name).await, StatusCode::NOT_FOUND);
}

async fn test_ctx_and_nar() -> (TestContext, NarFileName, Bytes) {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "cache");

    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;

    (test_ctx, NarFileName::from(&narinfo), nar_bytes)
}

async fn get_nar(
    test_ctx: &TestContext,
    nar_file_name: &NarFileName,
) -> StatusCode {
    let request = Request::builder()
        .uri(format!("/namespace/cache/nar/{nar_file_name}"))
        .body(Body::empty())
        .expect("valid request");

    let Ok(response) = test_ctx.app_router().call(request).await;

    response.status()
}

async fn put_nar(
    test_ctx: &mut TestContext,
    nar_file_name: &NarFileName,
    nar_bytes: Bytes,
) -> Response<Body> {
    let token = test_ctx.write_token("namespace", "cache");

    let request = Request::builder()
        .method(Method::PUT)
        .uri(format!("/namespace/cache/nar/{nar_file_name}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(nar_bytes))
        .expect("valid request");

    let Ok(response) = test_ctx.app_router().call(request).await;

    response
}
