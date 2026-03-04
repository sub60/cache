#![allow(incomplete_features)]
#![allow(missing_docs)]
#![feature(generic_const_exprs)]

mod common;

use core::net::Ipv4Addr;

use cache::types::NarInfoFileName;
use cache::{Context, ObjectStorage};
use common::{MockNixClient, NixClient, RealNixClient, TestContext};
use tokio::net::TcpListener;

use crate::common::NixStore;

#[tokio::test]
async fn copy_to_cache_mock() {
    let test_ctx = TestContext::default();

    let nix = MockNixClient::default();

    let path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([path.clone()])
        .to(test_ctx.app_router(), "user", "cache")
        .await
        .unwrap();

    assert!(
        test_ctx
            .object_storage()
            .contains_key(&NarInfoFileName { store_hash: path.into_hash() })
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn copy_to_cache_real() {
    let test_ctx = TestContext::default();

    // Start the cache server on a OS-assigned port.
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, test_ctx.app_router()).into_future());

    let nix = RealNixClient::new();

    let path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([path.clone()]).to(local_addr, "user", "cache").await.unwrap();

    assert!(
        test_ctx
            .object_storage()
            .contains_key(&NarInfoFileName { store_hash: path.into_hash() })
            .await
            .unwrap()
    );
}
