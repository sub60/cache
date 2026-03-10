use axum::http;
use cache::ObjectGateway;
use types::NarFileName;

use crate::R2ObjectStorage;

impl ObjectGateway for R2ObjectStorage {
    type Storage = Self;

    fn nar_url(&self, _key: &NarFileName) -> http::Uri {
        todo!()
    }

    fn storage(&self) -> &Self {
        self
    }
}
