use crate::cache_bucket::CacheBucket;
use crate::cdn::CloudflareCdn;
use crate::d1_cache_database::D1CacheDatabase;

#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) cache_bucket: CacheBucket,
    pub(crate) cache_db: D1CacheDatabase,
    pub(crate) cdn: CloudflareCdn,
}

impl sub60_cache::Context for Context {
    type CacheDb = D1CacheDatabase;
    type Cdn = CloudflareCdn;
    type ObjectGateway = CacheBucket;

    #[inline]
    fn cache_db(&self) -> &Self::CacheDb {
        &self.cache_db
    }

    #[inline]
    fn cdn(&self) -> &Self::Cdn {
        &self.cdn
    }

    #[inline]
    fn object_gateway(&self) -> &Self::ObjectGateway {
        &self.cache_bucket
    }
}
