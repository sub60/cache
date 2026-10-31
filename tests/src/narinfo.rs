use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use nix_types::{NarInfo, StorePath};
use tower_service::Service;

use crate::{MockNixClient, NixClient, NixStore, TestContext};

#[tokio::test]
async fn narinfo_is_only_served_by_the_cache_it_was_pushed_to() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "foo");
    test_ctx.db_mut().add_cache("namespace", "bar");

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(test_ctx.app_router(), "namespace", "foo")
        .await
        .unwrap();

    let mut app = test_ctx.app_router();

    let foo_response =
        get_narinfo(&mut app, "namespace/foo", &store_path).await;
    assert_eq!(foo_response.status(), StatusCode::OK);

    let bar_response =
        get_narinfo(&mut app, "namespace/bar", &store_path).await;
    assert_eq!(bar_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn namespace_lookup_ignores_case() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("NixOS");
    test_ctx.db_mut().add_cache("NixOS", "foo");

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(test_ctx.app_router(), "NixOS", "foo")
        .await
        .unwrap();

    let mut app = test_ctx.app_router();

    for namespace in ["NixOS", "nixos", "NIXOS"] {
        let response =
            get_narinfo(&mut app, &format!("{namespace}/foo"), &store_path)
                .await;
        assert_eq!(response.status(), StatusCode::OK, "{namespace}");
    }
}

#[tokio::test]
async fn each_cache_signs_its_own_copy_of_a_shared_store_path() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    let foo = test_ctx.db_mut().add_cache("namespace", "foo");
    let bar = test_ctx.db_mut().add_cache("namespace", "bar");

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    for cache in ["foo", "bar"] {
        nix.copy([store_path.clone()])
            .to(test_ctx.app_router(), "namespace", cache)
            .await
            .unwrap();
    }

    let mut app = test_ctx.app_router();

    let foo_narinfo = parse_narinfo(
        get_narinfo(&mut app, "namespace/foo", &store_path).await,
    )
    .await;

    let bar_narinfo = parse_narinfo(
        get_narinfo(&mut app, "namespace/bar", &store_path).await,
    )
    .await;

    let fingerprint = foo_narinfo.fingerprint().to_string();
    assert_eq!(bar_narinfo.fingerprint().to_string(), fingerprint);

    let foo_signature = foo.signing_key.sign(fingerprint.as_bytes());
    let bar_signature = bar.signing_key.sign(fingerprint.as_bytes());

    assert_eq!(&*foo_narinfo.signatures, &[foo_signature]);
    assert_eq!(&*bar_narinfo.signatures, &[bar_signature]);
}

#[tokio::test]
async fn narinfo_is_signed_with_the_current_signing_key() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_namespace("namespace");
    test_ctx.db_mut().add_cache("namespace", "foo");

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(test_ctx.app_router(), "namespace", "foo")
        .await
        .unwrap();

    let foo = test_ctx.db_mut().rotate_signing_key("namespace", "foo");

    let mut app = test_ctx.app_router();

    let narinfo = parse_narinfo(
        get_narinfo(&mut app, "namespace/foo", &store_path).await,
    )
    .await;

    let signature =
        foo.signing_key.sign(narinfo.fingerprint().to_string().as_bytes());

    assert_eq!(&*narinfo.signatures, &[signature]);
}

async fn get_narinfo(
    app: &mut axum::Router,
    base: &str,
    store_path: &StorePath,
) -> Response<Body> {
    let request = Request::builder()
        .uri(format!("/{base}/{}.narinfo", store_path.hash()))
        .body(Body::empty())
        .expect("valid request");

    let Ok(response) = app.call(request).await;

    response
}

async fn parse_narinfo(response: Response<Body>) -> NarInfo<String> {
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read narinfo body");

    core::str::from_utf8(&body)
        .expect("narinfo is valid UTF-8")
        .parse()
        .expect("narinfo is valid")
}
