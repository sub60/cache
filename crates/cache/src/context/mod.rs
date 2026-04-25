//! TODO: docs.

mod composed;

pub use composed::ComposedContext;

use crate::object_gateway::ObjectGateway;
use crate::{CacheDb, Cdn, ObjectStorage};

/// TODO: docs.
pub type MultipartUploadId<Ctx> =
    <<<Ctx as Context>::ObjectGateway as ObjectGateway>::Storage as ObjectStorage>::MultipartUploadId;

/// TODO: docs.
pub type ObjectStorageError<Ctx> =
    <<<Ctx as Context>::ObjectGateway as ObjectGateway>::Storage as ObjectStorage>::Error;

/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {
    /// The database storing all the caches' metadatas.
    type CacheDb: CacheDb;

    /// The CDN that sits in front of the cache server.
    type Cdn: Cdn;

    /// The gateway to the object storage that holds the NARs and NARInfos.
    type ObjectGateway: ObjectGateway;

    /// Returns a reference to the cache database.
    fn cache_db(&self) -> &Self::CacheDb;

    /// Returns a reference to the CDN.
    fn cdn(&self) -> &Self::Cdn;

    /// Returns a reference to the object gateway.
    fn object_gateway(&self) -> &Self::ObjectGateway;

    /// Returns a reference to the object storage.
    fn object_storage(
        &self,
    ) -> &<Self::ObjectGateway as ObjectGateway>::Storage {
        self.object_gateway().storage()
    }

    /// TODO: docs.
    fn want_mass_query(&self) -> bool {
        true
    }
}
