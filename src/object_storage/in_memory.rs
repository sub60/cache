use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;
use crate::types::{NarFileName, NarInfoFileName};

/// An [`ObjectStorage`] trait that stores the object in memory using a
/// `HashMap`.
#[derive(Clone, Default)]
pub struct InMemoryObjectStorage {
    objects: Arc<Mutex<HashMap<InMemoryObjectStorageKey, Vec<u8>>>>,
}

#[derive(PartialEq, Eq, Hash)]
enum InMemoryObjectStorageKey {
    Nar(NarFileName),
    NarInfo(NarInfoFileName),
}

#[expect(clippy::unwrap_used)]
impl ObjectStorage for InMemoryObjectStorage {
    type Error = core::convert::Infallible;

    async fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        Ok(self.objects.lock().unwrap().contains_key(&(key.into().into())))
    }

    async fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.objects.lock().unwrap().get(&(key.into().into())).cloned())
    }

    async fn set<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
        bytes: &[u8],
    ) -> Result<bool, Self::Error> {
        Ok(self
            .objects
            .lock()
            .unwrap()
            .insert(key.into().into(), bytes.to_vec())
            .is_none())
    }
}

impl From<&NarFileName> for InMemoryObjectStorageKey {
    fn from(nar_file_name: &NarFileName) -> Self {
        Self::Nar(nar_file_name.clone())
    }
}

impl From<&NarInfoFileName> for InMemoryObjectStorageKey {
    fn from(narinfo_file_name: &NarInfoFileName) -> Self {
        Self::NarInfo(narinfo_file_name.clone())
    }
}

impl From<ObjectStorageKey<'_>> for InMemoryObjectStorageKey {
    fn from(key: ObjectStorageKey<'_>) -> Self {
        match key {
            ObjectStorageKey::Nar(nar_file_name) => Self::from(nar_file_name),
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                Self::from(narinfo_file_name)
            },
        }
    }
}
