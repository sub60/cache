use core::marker::PhantomData;

use crate::{Context, ObjectGateway, ObjectStorage};

/// A composable [`Context`] implementation that simply holds the various
/// associated types as fields.
#[derive(Clone)]
pub struct ComposedContext<Cdn, ObjGateway, ObjStorage> {
    cdn: Cdn,
    object_gateway: ObjGateway,
    object_storage: PhantomData<ObjStorage>,
    want_mass_query: bool,
}

impl ComposedContext<(), (), ()> {
    /// Creates a new [`ComposedContext`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Cdn, ObjGateway, ObjStorage> ComposedContext<Cdn, ObjGateway, ObjStorage> {
    /// TODO: docs.
    pub fn with_cdn<NewCdn: crate::Cdn>(
        self,
        cdn: NewCdn,
    ) -> ComposedContext<NewCdn, ObjGateway, ObjStorage> {
        ComposedContext {
            cdn,
            object_gateway: self.object_gateway,
            object_storage: self.object_storage,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_object_gateway<NewObjGateway: ObjectGateway<ObjStorage>>(
        self,
        object_gateway: NewObjGateway,
    ) -> ComposedContext<Cdn, NewObjGateway, ObjStorage>
    where
        ObjStorage: ObjectStorage,
    {
        ComposedContext {
            cdn: self.cdn,
            object_gateway,
            object_storage: self.object_storage,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_object_storage<NewObjStorage: ObjectStorage>(
        self,
    ) -> ComposedContext<Cdn, ObjGateway, NewObjStorage> {
        ComposedContext {
            cdn: self.cdn,
            object_gateway: self.object_gateway,
            object_storage: PhantomData,
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
            cdn: (),
            object_gateway: (),
            object_storage: PhantomData,
            want_mass_query: true,
        }
    }
}

impl<Cdn, ObjGateway, ObjStorage> Context
    for ComposedContext<Cdn, ObjGateway, ObjStorage>
where
    Cdn: crate::Cdn + Clone + Send + Sync + 'static,
    ObjGateway: ObjectGateway<ObjStorage> + Clone + Send + Sync + 'static,
    ObjStorage: ObjectStorage + Clone + Send + Sync + 'static,
{
    type Cdn = Cdn;
    type ObjectGateway = ObjGateway;
    type ObjectStorage = ObjStorage;

    fn cdn(&self) -> &Self::Cdn {
        &self.cdn
    }

    fn object_gateway(&self) -> &Self::ObjectGateway {
        &self.object_gateway
    }

    fn want_mass_query(&self) -> bool {
        self.want_mass_query
    }
}
