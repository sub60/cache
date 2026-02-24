use core::convert::Infallible;

use axum::http;

use crate::types::{NarFileName, NixStoreHash};
use crate::{ObjectStorage, PresignRequest};

/// The [`ObjectStorage`] implementation for Cloudflare.
#[derive(Debug)]
pub struct CloudflareObjectStorage;

#[derive(Debug, Copy, Clone)]
pub enum CloudflareObjectStorageKey<'a> {
    Nar(&'a NarFileName),
    NarInfo(&'a NixStoreHash),
}

impl ObjectStorage for CloudflareObjectStorage {
    type Key<'a> = CloudflareObjectStorageKey<'a>;
    type Error = Infallible;

    async fn contains_key(
        &self,
        _key: Self::Key<'_>,
    ) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn get(
        &self,
        _key: Self::Key<'_>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        todo!()
    }

    fn object_url(&self, _key: Self::Key<'_>) -> http::Uri {
        todo!()
    }

    fn presign(
        &self,
        _presign_request: PresignRequest<Self::Key<'_>>,
    ) -> http::HeaderValue {
        todo!()
    }

    async fn set(
        &self,
        _key: Self::Key<'_>,
        _bytes: &[u8],
    ) -> Result<bool, Self::Error> {
        todo!()
    }
}

impl<'a> From<&'a NarFileName> for CloudflareObjectStorageKey<'a> {
    fn from(nar_file_name: &'a NarFileName) -> Self {
        Self::Nar(nar_file_name)
    }
}

impl<'a> From<&'a NixStoreHash> for CloudflareObjectStorageKey<'a> {
    fn from(store_hash: &'a NixStoreHash) -> Self {
        Self::NarInfo(store_hash)
    }
}
