use smol_str::SmolStr;
use types::{CacheName, CacheSigningKey, UserName};

#[derive(Clone)]
pub(crate) struct CacheCtx {
    /// The cache's name.
    #[expect(dead_code)]
    pub(crate) name: CacheName,
    /// The name of the user that owns the cache.
    #[expect(dead_code)]
    pub(crate) owner: UserName,
    /// The cache's `Priority` returned by the `/nix-cache-info` endpoint
    /// (usually 40).
    pub(crate) priority: u32,
    /// The key used to sign NARInfos.
    pub(crate) signing_key: CacheSigningKey,
    /// The cache's `StoreDir` returned by the `/nix-cache-info` endpoint
    /// (usually `"/nix/store"`).
    pub(crate) store_dir: SmolStr,
    #[expect(dead_code)]
    pub(crate) visibility: CacheVisibility,
}

#[derive(Copy, Clone)]
#[expect(dead_code)]
pub(crate) enum CacheVisibility {
    Public,
    Private,
}
