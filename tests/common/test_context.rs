use axum::Router;
use cache::cdn::{self, NoOpCdn};
use cache::context::ComposedContext;
use cache::object_gateway::IntoRouterObjectGateway;
use cache::object_storage::InMemoryObjectStorage;
use cache::{Context, ObjectGateway, ObjectStorage};

const OBJECT_GATEWAY_PATH: &str = "/objects";

/// A thin wrapper around a [`ComposedContext`] with convenient generic defaults
/// for testing.
///
/// It always uses a [`IntoRouterObjectGateway`] to serve objects from the
/// configured object storage, so it can be converted into a [`Router`] by
/// calling [`app_router()`](TestContext::app_router).
#[derive(Clone)]
pub struct TestContext<Cdn = NoOpCdn, ObjStorage = InMemoryObjectStorage> {
    inner: ComposedContext<Cdn, IntoRouterObjectGateway<ObjStorage>>,
}

impl<Cdn, ObjStorage> TestContext<Cdn, ObjStorage> {
    pub fn app_router(&self) -> Router
    where
        Cdn: cdn::Cdn,
        ObjStorage: Clone,
        IntoRouterObjectGateway<ObjStorage>: Into<Router>,
        ComposedContext<Cdn, IntoRouterObjectGateway<ObjStorage>>: Context<
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
    Cdn: cdn::Cdn + Clone,
    ObjStorage: ObjectStorage + Clone,
    IntoRouterObjectGateway<ObjStorage>: ObjectGateway + Clone,
    ComposedContext<Cdn, IntoRouterObjectGateway<ObjStorage>>:
        Context<Cdn = Cdn, ObjectGateway = IntoRouterObjectGateway<ObjStorage>>,
{
    type Cdn = Cdn;
    type ObjectGateway = IntoRouterObjectGateway<ObjStorage>;

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
