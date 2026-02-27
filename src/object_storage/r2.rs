use core::convert::Infallible;

use crate::ObjectStorage;
use crate::types::{NarFileName, NarInfoFileName};

/// The [`ObjectStorage`] implementation for R2.
#[derive(Debug)]
pub struct R2ObjectStorage;

#[derive(Debug, Copy, Clone)]
pub enum R2ObjectStorageKey<'a> {
    Nar(&'a NarFileName),
    NarInfo(&'a NarInfoFileName),
}

impl ObjectStorage for R2ObjectStorage {
    type Key<'a> = R2ObjectStorageKey<'a>;
    type Error = Infallible;

    async fn contains_key(
        &self,
        _key: &Self::Key<'_>,
    ) -> Result<bool, Self::Error> {
        todo!()
    }

    async fn get(
        &self,
        _key: &Self::Key<'_>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
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

impl<'a> From<&'a NarInfoFileName> for R2ObjectStorageKey<'a> {
    fn from(narinfo_file_name: &'a NarInfoFileName) -> Self {
        Self::NarInfo(narinfo_file_name)
    }
}
