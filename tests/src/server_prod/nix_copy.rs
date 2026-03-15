use crate::common::{NixClient, NixStore, RealNixClient};
use crate::server_prod::Workerd;

#[tokio::test]
#[cfg_attr(
    not(all(nix_in_path, workerd_in_path)),
    ignore = "requires both `nix` and `workerd` in $PATH"
)]
async fn copy_to_and_from_cache() {
    let workerd = Workerd::spawn();

    let nix = RealNixClient::new();

    let store_path = nix.store().add_file("hello-world", b"hello world").await;

    // Push to the cache served by workerd (backed by in-memory R2).
    nix.copy([store_path.clone()])
        .to(workerd.addr, "user", "cache")
        .await
        .unwrap();

    // Delete from the local Nix store.
    nix.store().delete(&store_path).await;
    assert!(!nix.store().contains(&store_path).await);

    // Pull from the cache.
    nix.copy([store_path.clone()])
        .from(workerd.addr, "user", "cache")
        .await
        .unwrap();

    assert!(nix.store().contains(&store_path).await);
}
