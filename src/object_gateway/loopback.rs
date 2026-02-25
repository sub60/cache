use core::fmt;
use core::str::FromStr;

use axum::http;

use crate::{ObjectGateway, ObjectStorage};

/// TODO: docs.
pub struct LoopbackObjectGateway<ObjStorage> {
    port: Option<u16>,
    storage: ObjStorage,
}

impl<ObjStorage> LoopbackObjectGateway<ObjStorage> {
    /// Creates a new [`LoopbackObjectGateway`] over the given object storage.
    pub fn new(storage: ObjStorage) -> Self {
        Self { port: None, storage }
    }

    /// Creates a new [`LoopbackObjectGateway`] over the given object storage.
    pub fn run(
        &self,
        _port: u16,
    ) -> impl Future<Output = ()> + use<ObjStorage> + Send
    where
        ObjStorage: ObjectStorage + Clone,
        for<'a> ObjStorage::Key<'a>: FromStr,
    {
        async {}
    }
}

impl<ObjStorage: ObjectStorage> ObjectGateway<ObjStorage>
    for LoopbackObjectGateway<ObjStorage>
where
    for<'a> ObjStorage::Key<'a>: fmt::Display,
{
    #[track_caller]
    fn object_url(&self, key: ObjStorage::Key<'_>) -> http::Uri {
        let Some(port) = self.port else {
            panic!(
                "LoopbackObjectGateway is not running. Please call \
                 `LoopbackObjectGateway::run()` and make sure the returned \
                 future is being polled"
            );
        };
        format!("http://localhost:{port}/{}", key).parse().expect("valid URI")
    }

    fn storage(&self) -> &ObjStorage {
        &self.storage
    }
}
