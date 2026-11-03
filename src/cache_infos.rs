use nix_types::CacheSigningKey;
use smol_str::SmolStr;

use crate::{CacheId, Permission, Permissions};

/// TODO: docs.
#[derive(Clone)]
pub struct CacheInfos {
    /// The cache's stable ID.
    pub id: CacheId,

    /// The cache's `Priority` returned by the `/nix-cache-info` endpoint
    /// (usually 40).
    pub priority: u32,

    /// What anyone can do to the cache, with or without a token.
    pub public_permissions: Permissions,

    /// The key used to sign NARInfos.
    pub signing_key: CacheSigningKey,

    /// The cache's `StoreDir` returned by the `/nix-cache-info` endpoint
    /// (usually `"/nix/store"`).
    pub store_dir: SmolStr,
}

impl CacheInfos {
    /// Returns whether anyone can pull from the cache, with or without a token.
    #[inline]
    pub fn is_public(&self) -> bool {
        self.public_permissions.contains(Permission::PullFromCache)
    }
}
