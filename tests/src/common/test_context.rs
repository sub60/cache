use core::fmt;

use axum::Router;
use sub60_cache::{Context, ObjectStorage};

use crate::common::{
    InMemoryCacheDb,
    InMemoryObjectStorage,
    IntoRouterObjectGateway,
    NoOpCdn,
    SystemClock,
};

const OBJECT_GATEWAY_PATH: &str = "/objects";

/// A thin wrapper around a [`ComposedContext`] with convenient generic defaults
/// for testing.
///
/// It always uses a [`IntoRouterObjectGateway`] to serve objects from the
/// configured object storage, so it can be converted into a [`Router`] by
/// calling [`app_router()`](TestContext::app_router).
#[derive(Clone)]
pub struct TestContext<Cdn = NoOpCdn, ObjStorage = InMemoryObjectStorage> {
    cache_db: InMemoryCacheDb,
    cdn: Cdn,
    clock: SystemClock,
    object_gateway: IntoRouterObjectGateway<ObjStorage>,
    want_mass_query: bool,
}

impl<Cdn, ObjStorage> TestContext<Cdn, ObjStorage> {
    /// TODO: docs.
    pub fn app_router(&self) -> Router
    where
        ObjStorage: Clone,
        IntoRouterObjectGateway<ObjStorage>: Into<Router>,
        Self: Clone + Context,
    {
        let object_gateway = self.object_gateway.clone();
        sub60_cache::router(self.clone()).merge(object_gateway)
    }

    /// TODO: docs.
    pub fn db_mut(&mut self) -> &mut InMemoryCacheDb {
        &mut self.cache_db
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            cache_db: Default::default(),
            cdn: Default::default(),
            clock: Default::default(),
            object_gateway: IntoRouterObjectGateway::new(
                OBJECT_GATEWAY_PATH,
                Default::default(),
            ),
            want_mass_query: true,
        }
    }
}

impl<Cdn, ObjStorage> Context for TestContext<Cdn, ObjStorage>
where
    Cdn: sub60_cache::Cdn + Clone + Send + Sync + 'static,
    ObjStorage: ObjectStorage + Clone + Send + Sync + 'static,
    ObjStorage::MultipartUploadId: fmt::Display,
{
    type CacheDb = InMemoryCacheDb;
    type Cdn = Cdn;
    type Clock = SystemClock;
    type ObjectGateway = IntoRouterObjectGateway<ObjStorage>;

    fn cache_db(&self) -> &Self::CacheDb {
        &self.cache_db
    }
    fn cdn(&self) -> &Self::Cdn {
        &self.cdn
    }
    fn clock(&self) -> &Self::Clock {
        &self.clock
    }
    fn object_gateway(&self) -> &Self::ObjectGateway {
        &self.object_gateway
    }
    fn want_mass_query(&self) -> bool {
        self.want_mass_query
    }
}
