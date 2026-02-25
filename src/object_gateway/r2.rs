use axum::http;

use crate::object_gateway::ObjectGateway;
use crate::object_storage::{ObjectStorage, R2ObjectStorage};

impl ObjectGateway for R2ObjectStorage {
    type Storage = Self;

    fn object_url(&self, _key: <Self as ObjectStorage>::Key<'_>) -> http::Uri {
        todo!()
    }

    fn storage(&self) -> &Self {
        self
    }
}
