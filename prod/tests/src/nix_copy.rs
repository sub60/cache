use sub60_cache_tests::{NixClient, NixStore, RealNixClient};

use crate::MiniflareServer;

#[tokio::test]
#[cfg_attr(not(nix_in_path), ignore = "requires `nix` in $PATH")]
async fn copy_to_and_from_cache() {
    let mf = MiniflareServer::spawn();

    let nix = RealNixClient::create();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    // Push to the cache served by miniflare (backed by local R2).
    nix.copy([store_path.clone()]).to(mf.addr, "user", "cache").await.unwrap();

    // Delete from the local Nix store.
    nix.store().delete(&store_path).await;
    assert!(!nix.store().contains(&store_path).await);

    // Pull from the cache.
    nix.copy([store_path.clone()])
        .from(mf.addr, "user", "cache")
        .await
        .unwrap();

    assert!(nix.store().contains(&store_path).await);
}
