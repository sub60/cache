use core::convert::Infallible;

use super::ObjectStorageKey;
use crate::ObjectStorage;

/// The [`ObjectStorage`] implementation for R2.
#[derive(Debug)]
pub struct R2ObjectStorage;

impl ObjectStorage for R2ObjectStorage {
    type Error = Infallible;

    async fn contains_key<'a>(
        &self,
        _key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn get<'a>(
        &self,
        _key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<impl AsRef<[u8]>>, Self::Error> {
        Ok(Some(vec![0u8; 10])) // TODO: return the actual bytes.
    }

    async fn set<'a>(
        &self,
        _key: impl Into<ObjectStorageKey<'a>>,
        _bytes: impl AsRef<[u8]>,
    ) -> Result<bool, Self::Error> {
        todo!()
    }
}
