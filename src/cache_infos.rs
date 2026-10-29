use nix_types::CacheSigningKey;
use smol_str::SmolStr;

use crate::CacheId;

/// TODO: docs.
#[derive(Clone)]
pub struct CacheInfos {
    /// The cache's stable ID.
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
