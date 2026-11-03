use core::time::Duration;

use axum::Router;
use getrandom::SysRng;
use rand_core::UnwrapErr;
use sub60_cache::{Context, TokenFormat};

use crate::common::{
    InMemoryCacheDb,
    InMemoryNarInfoStorage,
    InMemoryNarStorage,
    MockOidcVerifier,
    NoOpCdn,
    SystemClock,
    TestTokenFormat,
};

const NAR_STORAGE_PATH: &str = "/nars";

/// The origin the server is reachable at in tests.
pub const ORIGIN: &str = "https://cache.example.com";

/// A [`Context`] with convenient generic defaults for testing.
///
/// Its [`InMemoryNarStorage`] serves the NARs it stores, so it can be
/// converted into a [`Router`] by calling
/// [`app_router()`](TestContext::app_router).
#[derive(Clone)]
pub struct TestContext<Cdn = NoOpCdn> {
    cache_db: InMemoryCacheDb,
    cdn: Cdn,
    clock: SystemClock,
    nar_storage: InMemoryNarStorage,
    narinfo_storage: InMemoryNarInfoStorage,
    oidc_verifier: MockOidcVerifier,
    token_format: TestTokenFormat,
    want_mass_query: bool,
}

impl<Cdn> TestContext<Cdn> {
    /// TODO: docs.
    pub fn app_router(&self) -> Router
    where
        Self: Clone + Context,
    {
        let nar_storage = self.nar_storage.clone();
        sub60_cache::router(self.clone()).merge(nar_storage)
    }

    /// TODO: docs.
    pub fn db(&self) -> &InMemoryCacheDb {
        &self.cache_db
    }

    /// TODO: docs.
    pub fn db_mut(&mut self) -> &mut InMemoryCacheDb {
        &mut self.cache_db
    }

    /// Returns a token that can only write to the given cache.
    ///
    /// # Panics
    ///
    /// Panics if the cache does not exist.
    #[track_caller]
    pub fn write_token(&self, namespace: &str, cache: &str) -> String {
        let token =
            self.cache_db.add_ci_job_token(namespace, cache, Duration::MAX);
        self.token_format.display(&token).to_string()
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            cache_db: Default::default(),
            cdn: Default::default(),
            clock: Default::default(),
            nar_storage: InMemoryNarStorage::new(NAR_STORAGE_PATH),
            narinfo_storage: Default::default(),
            oidc_verifier: Default::default(),
            token_format: Default::default(),
            want_mass_query: true,
        }
    }
}

impl<Cdn> Context for TestContext<Cdn>
where
    Cdn: sub60_cache::Cdn + Clone + Send + Sync + 'static,
{
    type CacheDb = InMemoryCacheDb;
    type Cdn = Cdn;
    type Clock = SystemClock;
    type NarInfoStorage = InMemoryNarInfoStorage;
    type NarStorage = InMemoryNarStorage;
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
    fn nar_storage(&self) -> &Self::NarStorage {
        &self.nar_storage
    }
    fn narinfo_storage(&self) -> &Self::NarInfoStorage {
        &self.narinfo_storage
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
