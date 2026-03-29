use axum::Router;
use cache::cdn::{self, NoOpCdn};
use cache::context::ComposedContext;
use cache::object_gateway::IntoRouterObjectGateway;
use cache::object_storage::InMemoryObjectStorage;
use cache::{Context, ObjectStorage};

use crate::common::InMemoryCacheDatabase;

const OBJECT_GATEWAY_PATH: &str = "/objects";

/// A thin wrapper around a [`ComposedContext`] with convenient generic defaults
/// for testing.
///
/// It always uses a [`IntoRouterObjectGateway`] to serve objects from the
/// configured object storage, so it can be converted into a [`Router`] by
/// calling [`app_router()`](TestContext::app_router).
#[derive(Clone)]
pub(crate) struct TestContext<Cdn = NoOpCdn, ObjStorage = InMemoryObjectStorage>
{
    inner: ComposedContext<
        InMemoryCacheDatabase,
        Cdn,
        IntoRouterObjectGateway<ObjStorage>,
    >,
}

impl<Cdn, ObjStorage> TestContext<Cdn, ObjStorage> {
    pub fn app_router(&self) -> Router
    where
        Cdn: cdn::Cdn,
        ObjStorage: Clone,
        IntoRouterObjectGateway<ObjStorage>: Into<Router>,
        ComposedContext<
            InMemoryCacheDatabase,
            Cdn,
            IntoRouterObjectGateway<ObjStorage>,
        >: Context<
                Cdn = Cdn,
                ObjectGateway = IntoRouterObjectGateway<ObjStorage>,
            >,
    {
        let object_gateway = self.inner.object_gateway().clone();
        cache::router(self.inner.clone()).merge(object_gateway)
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            inner: ComposedContext::new()
                .with_cache_db(Default::default())
                .with_cdn(Default::default())
                .with_object_gateway(IntoRouterObjectGateway::new(
                    OBJECT_GATEWAY_PATH,
                    Default::default(),
                )),
        }
    }
}

impl<Cdn, ObjStorage> Context for TestContext<Cdn, ObjStorage>
where
    Cdn: cdn::Cdn + Clone + Send + Sync + 'static,
    ObjStorage: ObjectStorage + Clone + Send + Sync + 'static,
{
    type CacheDb = InMemoryCacheDatabase;
    type Cdn = Cdn;
    type ObjectGateway = IntoRouterObjectGateway<ObjStorage>;

    fn cache_db(&self) -> &Self::CacheDb {
        self.inner.cache_db()
    }
    fn cdn(&self) -> &Self::Cdn {
        self.inner.cdn()
    }
    fn object_gateway(&self) -> &Self::ObjectGateway {
        self.inner.object_gateway()
    }
    fn want_mass_query(&self) -> bool {
        self.inner.want_mass_query()
    }
}
