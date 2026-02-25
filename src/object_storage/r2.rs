use core::convert::Infallible;

use axum::http;

use crate::ObjectStorage;
use crate::types::{NarFileName, NixStoreHash};

/// The [`ObjectStorage`] implementation for R2.
#[derive(Debug)]
pub struct R2ObjectStorage;

#[derive(Debug, Copy, Clone)]
pub enum R2ObjectStorageKey<'a> {
    Nar(&'a NarFileName),
    NarInfo(&'a NixStoreHash),
}

impl ObjectStorage for R2ObjectStorage {
    type Key<'a> = R2ObjectStorageKey<'a>;
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

    async fn set(
        &self,
        _key: Self::Key<'_>,
        _bytes: &[u8],
    ) -> Result<bool, Self::Error> {
        todo!()
    }
}

impl<'a> From<&'a NarFileName> for R2ObjectStorageKey<'a> {
    fn from(nar_file_name: &'a NarFileName) -> Self {
        Self::Nar(nar_file_name)
    }
}

impl<'a> From<&'a NixStoreHash> for R2ObjectStorageKey<'a> {
    fn from(store_hash: &'a NixStoreHash) -> Self {
        Self::NarInfo(store_hash)
    }
}
