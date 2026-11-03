use sub60_cache::CacheControl;

/// A [`CacheControl`] that keeps all the defaults.
#[derive(Debug, Copy, Clone, Default)]
pub struct DefaultCacheControl;

impl CacheControl for DefaultCacheControl {}
