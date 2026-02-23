use either::Either;
use smol_str::SmolStr;

use crate::types::{CacheName, NixStoreLiteral};

#[derive(Clone)]
#[expect(dead_code)]
pub(crate) struct CacheCtx {
    pub(crate) name: CacheName,
    pub(crate) priority: u32,
    pub(crate) store_dir: Either<NixStoreLiteral, SmolStr>,
}
