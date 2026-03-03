mod common;

use core::net::Ipv4Addr;

use axum::Router;
use cache::Context;
use common::{MockNixClient, NixClient, RealNixClient, TestContext};
use tokio::net::TcpListener;

#[tokio::test]
async fn mock_nix_copy_to() {
    let test_ctx = TestContext::default();
    let nar = "hello world";
    MockNixClient
        .copy([nar])
        .to(Router::from(test_ctx.clone()), "user", "cache")
        .await
        .unwrap();
    let obj_storage = test_ctx.object_storage();
    assert_eq!(
        obj_storage.get(&nar.into()).await.unwrap().unwrap(),
        nar.as_bytes()
    );
}

#[tokio::test]
async fn real_nix_copy_to() {
    let test_ctx = TestContext::default();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, Router::from(test_ctx.clone())));
    RealNixClient
        .copy(["nixpkgs#hello"])
        .to(local_addr, "user", "cache")
        .await
        .unwrap();
    let obj_storage = test_ctx.object_storage();
    assert_eq!(
        obj_storage.get(&nar.into()).await.unwrap().unwrap(),
        nar.as_bytes()
    );
}
