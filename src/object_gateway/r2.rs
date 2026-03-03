use axum::http;

use crate::object_gateway::ObjectGateway;
use crate::object_storage::R2ObjectStorage;
use crate::types::NarFileName;

impl ObjectGateway for R2ObjectStorage {
    type Storage = Self;

    fn nar_url(&self, _key: &NarFileName) -> http::Uri {
        todo!()
    }

    fn storage(&self) -> &Self {
        self
    }
}
