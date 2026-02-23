use smol_str::SmolStr;

use crate::types::CacheName;

#[derive(Clone)]
#[expect(dead_code)]
pub(crate) struct CacheCtx {
    pub(crate) name: CacheName,
    pub(crate) priority: u32,
    pub(crate) store_dir: SmolStr,
}
