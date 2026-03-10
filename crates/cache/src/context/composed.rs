use crate::{Context, ObjectGateway};

/// A composable [`Context`] implementation that simply holds the various
/// associated types as fields.
#[derive(Clone)]
pub struct ComposedContext<Cdn, ObjGateway> {
    cdn: Cdn,
    object_gateway: ObjGateway,
    want_mass_query: bool,
}

impl ComposedContext<(), ()> {
    /// Creates a new [`ComposedContext`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Cdn, ObjGateway> ComposedContext<Cdn, ObjGateway> {
    /// TODO: docs.
    pub fn with_cdn<NewCdn: crate::Cdn>(
        self,
        cdn: NewCdn,
    ) -> ComposedContext<NewCdn, ObjGateway> {
        ComposedContext {
            cdn,
            object_gateway: self.object_gateway,
            want_mass_query: self.want_mass_query,
        }
    }

    /// TODO: docs.
    pub fn with_object_gateway<NewObjGateway>(
        self,
        object_gateway: NewObjGateway,
    ) -> ComposedContext<Cdn, NewObjGateway> {
        ComposedContext {
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

impl Default for ComposedContext<(), ()> {
    fn default() -> Self {
        Self { cdn: (), object_gateway: (), want_mass_query: true }
    }
}

impl<Cdn, ObjGateway> Context for ComposedContext<Cdn, ObjGateway>
where
    Cdn: crate::Cdn,
    ObjGateway: ObjectGateway,
    Self: Clone + Send + Sync + 'static,
{
    type Cdn = Cdn;
    type ObjectGateway = ObjGateway;

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
