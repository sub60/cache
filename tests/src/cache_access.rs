use core::time::Duration;

use axum::body::Body;
use axum::http::{Method, Request, Response, StatusCode, header};
use base64::prelude::*;
use nix_types::NarFileName;
use sub60_cache::{Clock, Context, TokenFormat};
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn writes_need_a_token() {
    let test_ctx = test_ctx();

    let response = put_nar(&test_ctx, "foo", None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::WWW_AUTHENTICATE], "Bearer");
}

#[tokio::test]
async fn writes_with_a_token_for_the_cache_are_allowed() {
    let test_ctx = test_ctx();
    let token = test_ctx.write_token("namespace", "foo");

    let response =
        put_nar(&test_ctx, "foo", Some(&format!("Bearer {token}"))).await;
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
}

#[tokio::test]
async fn tokens_can_be_sent_as_basic_credentials() {
    let test_ctx = test_ctx();
    let token = test_ctx.write_token("namespace", "foo");
    let credentials = BASE64_STANDARD.encode(format!("sub60:{token}"));

    let response =
        put_nar(&test_ctx, "foo", Some(&format!("Basic {credentials}"))).await;
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
}

#[tokio::test]
async fn tokens_are_checked_against_the_current_time() {
    let test_ctx = test_ctx();
    let now = test_ctx.clock().unix_ts();
    let hour = Duration::from_secs(3600);

    for (expires_at, status) in [
        (now - hour, StatusCode::UNAUTHORIZED),
        (now + hour, StatusCode::TEMPORARY_REDIRECT),
    ] {
        let token =
            test_ctx.db().add_ci_job_token("namespace", "foo", expires_at);
        let token = test_ctx.token_format().display(&token).to_string();

        let response =
            put_nar(&test_ctx, "foo", Some(&format!("Bearer {token}"))).await;
        assert_eq!(response.status(), status, "{expires_at:?}");
    }
}

#[tokio::test]
async fn malformed_authorization_headers_are_unauthorized() {
    let test_ctx = test_ctx();
    let token = test_ctx.write_token("namespace", "foo");

    for authorization in [
        "Bearer not-a-token".to_owned(),
        format!("Token {token}"),
        format!("Basic {token}"),
        token,
    ] {
        let response = put_nar(&test_ctx, "foo", Some(&authorization)).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{authorization}"
        );
    }

    let response =
        get_nix_cache_info(&test_ctx, "foo", Some("Bearer nope")).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn public_caches_can_be_read_without_a_token() {
    let test_ctx = test_ctx();

    let response = get_nix_cache_info(&test_ctx, "foo", None).await;
    assert_eq!(response.status(), StatusCode::OK);
}

fn test_ctx() -> TestContext {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "foo");
    test_ctx
}

async fn put_nar(
    test_ctx: &TestContext,
    cache: &str,
    authorization: Option<&str>,
) -> Response<Body> {
    let nix = MockNixClient::default();
    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let (narinfo, _) = nix.store().nar_info_and_bytes(&store_path).await;
    let nar_file_name = NarFileName::from(&narinfo);

    send(
        test_ctx,
        Method::PUT,
        &format!("/namespace/{cache}/nar/{nar_file_name}"),
        authorization,
    )
    .await
}

async fn get_nix_cache_info(
    test_ctx: &TestContext,
    cache: &str,
    authorization: Option<&str>,
) -> Response<Body> {
    send(
        test_ctx,
        Method::GET,
        &format!("/namespace/{cache}/nix-cache-info"),
        authorization,
    )
    .await
}

async fn send(
    test_ctx: &TestContext,
    method: Method,
    uri: &str,
    authorization: Option<&str>,
) -> Response<Body> {
    let mut request = Request::builder().method(method).uri(uri);

    if let Some(authorization) = authorization {
        request = request.header(header::AUTHORIZATION, authorization);
    }

    let request = request.body(Body::empty()).expect("valid request");

    let Ok(response) = test_ctx.app_router().call(request).await;

    response
}
