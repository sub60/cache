#![allow(incomplete_features)]
#![allow(missing_docs)]
#![feature(generic_const_exprs)]

mod common;

use core::net::Ipv4Addr;

use axum::Router;
use cache::types::NarFileName;
use cache::{Context, ObjectStorage};
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
    let nar_filename = "todo".parse::<NarFileName>().unwrap();
    assert_eq!(
        obj_storage.get(&nar_filename).await.unwrap().unwrap(),
        nar.as_bytes()
    );
}

#[tokio::test]
async fn real_nix_copy_to() {
    let test_ctx = TestContext::default();
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(
        axum::serve(listener, Router::from(test_ctx.clone())).into_future(),
    );
    RealNixClient::new()
        .copy(["nixpkgs#hello"])
        .to(local_addr, "user", "cache")
        .await
        .unwrap();
    let _obj_storage = test_ctx.object_storage();
    todo!("assert something about the object storage");
}
