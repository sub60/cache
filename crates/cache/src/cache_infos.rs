use smol_str::SmolStr;
use types::CacheSigningKey;

/// TODO: docs.
#[derive(Clone)]
pub struct CacheInfos<CacheId> {
    /// The cache's ID in the database.
    pub id: CacheId,

    /// The cache's `Priority` returned by the `/nix-cache-info` endpoint
    /// (usually 40).
    pub priority: u32,

    /// The key used to sign NARInfos.
    pub signing_key: CacheSigningKey,

    /// The cache's `StoreDir` returned by the `/nix-cache-info` endpoint
    /// (usually `"/nix/store"`).
    pub store_dir: SmolStr,
}
