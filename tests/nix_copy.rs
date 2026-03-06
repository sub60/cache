#![allow(incomplete_features)]
#![allow(missing_docs)]
#![feature(generic_const_exprs)]

mod common;

use core::net::Ipv4Addr;

use cache::types::{NarFileName, NarInfoFileName};
use cache::{Context, ObjectStorage};
use common::{MockNixClient, NixClient, RealNixClient, TestContext};
use tokio::net::TcpListener;

use crate::common::NixStore;

#[tokio::test]
async fn copy_to_cache_mock() {
    let test_ctx = TestContext::default();

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(test_ctx.app_router(), "user", "cache")
        .await
        .unwrap();

    let narinfo_key = NarInfoFileName { store_hash: store_path.into_hash() };

    assert!(
        test_ctx.object_storage().contains_key(&narinfo_key).await.unwrap()
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

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(local_addr, "user", "cache")
        .await
        .unwrap();

    let narinfo_key = NarInfoFileName { store_hash: store_path.into_hash() };

    assert!(
        test_ctx.object_storage().contains_key(&narinfo_key).await.unwrap()
    );
}

#[tokio::test]
async fn copy_from_cache_mock() {
    let test_ctx = TestContext::default();

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;
    let narinfo = nix.store().narinfo(&store_path).await;
    let nar_bytes = nix.store().nar_bytes(&store_path).await;

    let nar_key = NarFileName { file_hash: narinfo.file_hash, extension: None };
    let narinfo_key = NarInfoFileName { store_hash: *store_path.hash() };

    // Pre-populate the object storage with the NAR and NARInfo so the cache
    // server can serve them.
    let obj_storage = test_ctx.object_storage();
    obj_storage.set(&nar_key, nar_bytes).await.unwrap();
    obj_storage.set(&narinfo_key, narinfo.to_string()).await.unwrap();

    // Remove the store path force Nix to fetch from the cache.
    nix.store().remove_store_path(&store_path).await;
    assert!(!nix.store().contains(&store_path).await);

    nix.copy([store_path.clone()])
        .from(test_ctx.app_router(), "user", "cache")
        .await
        .unwrap();

    assert!(nix.store().contains(&store_path).await);
}
