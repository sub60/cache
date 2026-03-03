use axum::Router;
use cache::cdn::{self, NoOpCdn};
use cache::context::ComposedContext;
use cache::object_gateway::IntoRouterObjectGateway;
use cache::object_storage::InMemoryObjectStorage;
use cache::{Context, ObjectStorage};

const OBJECT_GATEWAY_PATH: &str = "/objects";

/// A thin wrapper around a [`ComposedContext`] with convenient generic defaults
/// for testing.
///
/// It always uses a [`IntoRouterObjectGateway`] to serve objects from the
/// configured object storage, so it can be converted into a [`Router`] via its
/// [`Into<Router>`] implementation.
#[derive(Clone)]
pub struct TestContext<Cdn = NoOpCdn, ObjStorage = InMemoryObjectStorage> {
    inner: ComposedContext<Cdn, IntoRouterObjectGateway<ObjStorage>>,
}

impl<Cdn: cdn::Cdn + Default, ObjStorage: Default> Default
    for TestContext<Cdn, ObjStorage>
{
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

impl<Cdn, ObjStorage> From<TestContext<Cdn, ObjStorage>> for Router
where
    Cdn: cdn::Cdn,
    ObjStorage: Clone,
    IntoRouterObjectGateway<ObjStorage>: Into<Router>,
{
    fn from(ctx: TestContext<Cdn, ObjStorage>) -> Self {
        let object_gateway = ctx.inner.object_gateway().clone();
        cache::router(ctx.inner).merge(object_gateway)
    }
}

impl<Cdn, ObjStorage> Context for TestContext<Cdn, ObjStorage>
where
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
