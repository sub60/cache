use core::fmt;

use axum::Router;
use getrandom::SysRng;
use rand_core::UnwrapErr;
use sub60_cache::{Context, ObjectStorage};

use crate::common::{
    InMemoryCacheDb,
    InMemoryObjectStorage,
    IntoRouterObjectGateway,
    MockOidcVerifier,
    NoOpCdn,
    SystemClock,
    TestTokenFormat,
};

const OBJECT_GATEWAY_PATH: &str = "/objects";

/// The origin the server is reachable at in tests.
pub const ORIGIN: &str = "https://cache.example.com";

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
    oidc_verifier: MockOidcVerifier,
    token_format: TestTokenFormat,
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
    pub fn db(&self) -> &InMemoryCacheDb {
        &self.cache_db
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
            oidc_verifier: Default::default(),
            token_format: Default::default(),
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
    type OidcVerifier = MockOidcVerifier;
    type Rng = UnwrapErr<SysRng>;
    type TokenFormat = TestTokenFormat;

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
    fn oidc_verifier(&self) -> &Self::OidcVerifier {
        &self.oidc_verifier
    }
    fn origin(&self) -> &str {
        ORIGIN
    }
    fn rng(&self) -> Self::Rng {
        UnwrapErr(SysRng)
    }
    fn token_format(&self) -> &Self::TokenFormat {
        &self.token_format
    }
    fn want_mass_query(&self) -> bool {
        self.want_mass_query
    }
}
