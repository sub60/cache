//! TODO: docs.

mod composed;

pub use composed::ComposedContext;

use crate::object_gateway::ObjectGateway;
use crate::{Cdn, ObjectStorage};

/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {
    /// The CDN that sits in front of the cache server.
    type Cdn: Cdn;

    /// The object storage that holds the NARs and NARInfos.
    type ObjectGateway: ObjectGateway<Self::ObjectStorage>;

    /// The object storage that holds the NARs and NARInfos.
    type ObjectStorage: ObjectStorage;

    /// Returns a reference to the CDN.
    fn cdn(&self) -> &Self::Cdn;

    /// Returns a reference to the object gateway.
    fn object_gateway(&self) -> &Self::ObjectGateway;

    /// Returns a reference to the object storage.
    fn object_storage(&self) -> &Self::ObjectStorage {
        self.object_gateway().storage()
    }

    /// TODO: docs.
    fn want_mass_query(&self) -> bool {
        true
    }
}
