use crate::{CacheDb, Context, ObjectGateway};

/// A composable [`Context`] implementation that simply holds the various
/// associated types as fields.
#[derive(Clone)]
pub struct ComposedContext<Db, Cdn, ObjGateway> {
    cache_db: Db,
    cdn: Cdn,
    object_gateway: ObjGateway,
    want_mass_query: bool,
}

impl ComposedContext<(), (), ()> {
    /// Creates a new [`ComposedContext`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Db, Cdn, ObjGateway> ComposedContext<Db, Cdn, ObjGateway> {
    /// Returns a mutable reference to the cache database.
    pub fn cache_db_mut(&mut self) -> &mut Db {
        &mut self.cache_db
    }

    /// TODO: docs.
    pub fn with_cache_db<NewDb: CacheDb>(
        self,
        cache_db: NewDb,
    ) -> ComposedContext<NewDb, Cdn, ObjGateway> {
        ComposedContext {
            cache_db,
            cdn: self.cdn,
            object_gateway: self.object_gateway,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_cdn<NewCdn: crate::Cdn>(
        self,
        cdn: NewCdn,
    ) -> ComposedContext<Db, NewCdn, ObjGateway> {
        ComposedContext {
            cache_db: self.cache_db,
            cdn,
            object_gateway: self.object_gateway,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_object_gateway<NewObjGateway>(
        self,
        object_gateway: NewObjGateway,
    ) -> ComposedContext<Db, Cdn, NewObjGateway> {
        ComposedContext {
            cache_db: self.cache_db,
            cdn: self.cdn,
            object_gateway,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_want_mass_query(mut self, want_mass_query: bool) -> Self {
        self.want_mass_query = want_mass_query;
        self
    }
}

impl Default for ComposedContext<(), (), ()> {
    fn default() -> Self {
        Self {
            cache_db: (),
            cdn: (),
            object_gateway: (),
            want_mass_query: true,
        }
    }
}

impl<Db, Cdn, ObjGateway> Context for ComposedContext<Db, Cdn, ObjGateway>
where
    Db: CacheDb,
    Cdn: crate::Cdn,
    ObjGateway: ObjectGateway,
    Self: Clone + Send + Sync + 'static,
{
    type CacheDb = Db;
    type Cdn = Cdn;
    type ObjectGateway = ObjGateway;

    #[inline(always)]
    fn cache_db(&self) -> &Self::CacheDb {
        &self.cache_db
    }

    #[inline(always)]
    fn cdn(&self) -> &Self::Cdn {
        &self.cdn
    }

    #[inline(always)]
    fn object_gateway(&self) -> &Self::ObjectGateway {
        &self.object_gateway
    }

    #[inline(always)]
    fn want_mass_query(&self) -> bool {
        self.want_mass_query
    }
}
