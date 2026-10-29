use core::net::Ipv4Addr;

use nix_types::{NarFileName, NarInfoFileName};
use sub60_cache::{Context, ObjectStorage, ObjectStorageKey};
use tokio::net::TcpListener;

use crate::{MockNixClient, NixClient, NixStore, RealNixClient, TestContext};

#[tokio::test]
async fn copy_to_cache_mock() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    let cache = test_ctx.db_mut().add_cache("user", "cache");

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(test_ctx.app_router(), "user", "cache")
        .await
        .unwrap();

    let narinfo_key = NarInfoFileName { store_hash: store_path.into_hash() };

    assert!(
        test_ctx
            .object_storage()
            .contains_key(ObjectStorageKey::NarInfo(&cache.id, &narinfo_key))
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn copy_to_cache_real() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    let cache = test_ctx.db_mut().add_cache("user", "cache");

    // Start the cache server on a OS-assigned port.
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, test_ctx.app_router()).into_future());

    let nix = RealNixClient::create();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    nix.copy([store_path.clone()])
        .to(local_addr, "user", "cache")
        .await
        .unwrap();

    let narinfo_key = NarInfoFileName { store_hash: store_path.into_hash() };

    assert!(
        test_ctx
            .object_storage()
            .contains_key(ObjectStorageKey::NarInfo(&cache.id, &narinfo_key))
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn copy_from_cache_mock() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    let cache = test_ctx.db_mut().add_cache("user", "cache");

    let nix = MockNixClient::default();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;

    let nar_key = NarFileName::from(&narinfo);
    let narinfo_key = NarInfoFileName { store_hash: *store_path.hash() };

    // Pre-populate the object storage with the NAR and NARInfo so the cache
    // server can serve them.
    let obj_storage = test_ctx.object_storage();
    obj_storage
        .insert(ObjectStorageKey::Nar(&nar_key), nar_bytes)
        .await
        .unwrap();
    obj_storage
        .insert(
            ObjectStorageKey::NarInfo(&cache.id, &narinfo_key),
            narinfo.to_string(),
        )
        .await
        .unwrap();

    // Remove the store path force Nix to fetch from the cache.
    nix.store().delete(&store_path).await;
    assert!(!nix.store().contains(&store_path).await);

    nix.copy([store_path.clone()])
        .from(test_ctx.app_router(), "user", "cache")
        .await
        .unwrap();

    assert!(nix.store().contains(&store_path).await);
}

#[tokio::test]
async fn copy_from_cache_real() {
    let mut test_ctx = TestContext::default();
    test_ctx.db_mut().add_user("user");
    let cache = test_ctx.db_mut().add_cache("user", "cache");

    // Start the cache server on a OS-assigned port.
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    tokio::spawn(axum::serve(listener, test_ctx.app_router()).into_future());

    let nix = RealNixClient::create();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    let (narinfo, nar_bytes) =
        nix.store().nar_info_and_bytes(&store_path).await;

    let nar_key = NarFileName::from(&narinfo);
    let narinfo_key = NarInfoFileName { store_hash: *store_path.hash() };

    // Pre-populate the object storage with the NAR and NARInfo so the cache
    // server can serve them.
    let obj_storage = test_ctx.object_storage();
    obj_storage
        .insert(ObjectStorageKey::Nar(&nar_key), nar_bytes)
        .await
        .unwrap();
    obj_storage
        .insert(
            ObjectStorageKey::NarInfo(&cache.id, &narinfo_key),
            narinfo.to_string(),
        )
        .await
        .unwrap();

    // Remove the store path force Nix to fetch from the cache.
    nix.store().delete(&store_path).await;
    assert!(!nix.store().contains(&store_path).await);

    nix.copy([store_path.clone()])
        .from(local_addr, "user", "cache")
        .await
        .unwrap();

    assert!(nix.store().contains(&store_path).await);
}
