#![allow(incomplete_features)]
#![allow(missing_docs)]
#![feature(generic_const_exprs)]

mod common;

use core::net::Ipv4Addr;

use cache::types::{NarFileName, NarInfoFileName};
use cache::{Context, ObjectStorage};
use common::{MockNixClient, NixClient, RealNixClient, TestContext};
use tokio::net::TcpListener;

#[tokio::test]
async fn mock_nix_copy_to() {
    let test_ctx = TestContext::default();
    let nar = "hello world";
    MockNixClient
        .copy([nar])
        .to(test_ctx.app_router(), "user", "cache")
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

    // Start the cache server on a OS-assigned port.
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, test_ctx.app_router()).into_future());

    let nix = RealNixClient::new();

    let path = nix.add_file("hello-world", b"hello world");

    nix.copy([path.clone()]).to(local_addr, "user", "cache").await.unwrap();

    assert!(
        test_ctx
            .object_storage()
            .contains_key(&NarInfoFileName { store_hash: path.into_hash() })
            .await
            .unwrap()
    );
}
