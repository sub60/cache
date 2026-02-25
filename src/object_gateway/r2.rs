use axum::http;

use crate::object_gateway::ObjectGateway;
use crate::object_storage::{ObjectStorage, R2ObjectStorage};

impl ObjectGateway<Self> for R2ObjectStorage {
    fn object_url(&self, _key: <Self as ObjectStorage>::Key<'_>) -> http::Uri {
        todo!()
    }

    fn storage(&self) -> &Self {
        self
    }
}
