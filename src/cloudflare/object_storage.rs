use core::convert::Infallible;

use axum::http;

use crate::types::{NarFileName, NixStoreHash};
use crate::{ObjectStorage, PresignRequest};

/// The [`ObjectStorage`] implementation for Cloudflare.
#[derive(Debug)]
pub struct CloudflareObjectStorage;

#[derive(Debug)]
pub enum CloudflareObjectStorageKey {
    Nar(NarFileName),
    NarInfo(NixStoreHash),
}

impl ObjectStorage for CloudflareObjectStorage {
    type Key = CloudflareObjectStorageKey;
    type Error = Infallible;

    async fn contains_key(
        &self,
        _key: &Self::Key,
    ) -> Result<bool, Self::Error> {
        todo!()
    }

    fn presign(
        &self,
        _presign_request: PresignRequest<Self::Key>,
    ) -> http::HeaderValue {
        todo!()
    }
}

impl From<NarFileName> for CloudflareObjectStorageKey {
    fn from(nar_file_name: NarFileName) -> Self {
        Self::Nar(nar_file_name)
    }
}

impl From<NixStoreHash> for CloudflareObjectStorageKey {
    fn from(store_hash: NixStoreHash) -> Self {
        Self::NarInfo(store_hash)
    }
}
