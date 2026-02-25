use crate::Context;

/// A composable [`Context`] implementation that simply holds the various
/// associated types as fields.
#[derive(Clone)]
pub struct ComposedContext<Cdn, ObjStorage> {
    cdn: Cdn,
    object_storage: ObjStorage,
    want_mass_query: bool,
}

impl ComposedContext<(), ()> {
    /// Creates a new [`ComposedContext`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Cdn, ObjStorage> ComposedContext<Cdn, ObjStorage> {
    /// TODO: docs.
    pub fn with_cdn<NewCdn>(
        self,
        cdn: NewCdn,
    ) -> ComposedContext<NewCdn, ObjStorage> {
        ComposedContext {
            cdn,
            object_storage: self.object_storage,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_object_storage<NewObjStorage>(
        self,
        object_storage: NewObjStorage,
    ) -> ComposedContext<Cdn, NewObjStorage> {
        ComposedContext {
            cdn: self.cdn,
            object_storage,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_want_mass_query(mut self, want_mass_query: bool) -> Self {
        self.want_mass_query = want_mass_query;
        self
    }
}

impl Default for ComposedContext<(), ()> {
    fn default() -> Self {
        Self { cdn: (), object_storage: (), want_mass_query: true }
    }
}

impl<Cdn, ObjStorage> Context for ComposedContext<Cdn, ObjStorage>
where
    Cdn: crate::Cdn + Clone + Send + Sync + 'static,
    ObjStorage: crate::ObjectStorage + Clone + Send + Sync + 'static,
{
    type Cdn = Cdn;
    type ObjectStorage = ObjStorage;

    fn cdn(&self) -> &Self::Cdn {
        &self.cdn
    }

    fn object_storage(&self) -> &Self::ObjectStorage {
        &self.object_storage
    }

    fn want_mass_query(&self) -> bool {
        self.want_mass_query
    }
}
