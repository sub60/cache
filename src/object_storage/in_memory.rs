use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;
use crate::types::{NarFileName, NarInfoFileName};

/// An [`ObjectStorage`] trait that stores the object in memory using a
/// `HashMap`.
#[derive(Clone, Default)]
pub struct InMemoryObjectStorage {
    nars: Arc<Mutex<HashMap<NarFileName, Bytes>>>,
    nar_infos: Arc<Mutex<HashMap<NarInfoFileName, Bytes>>>,
}

#[expect(clippy::unwrap_used)]
impl ObjectStorage for InMemoryObjectStorage {
    type Error = core::convert::Infallible;

    async fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        Ok(match key.into() {
            ObjectStorageKey::Nar(nar_file_name) => {
                self.nars.lock().unwrap().contains_key(nar_file_name)
            },
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                self.nar_infos.lock().unwrap().contains_key(narinfo_file_name)
            },
        })
    }

    async fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<Bytes>, Self::Error> {
        Ok(match key.into() {
            ObjectStorageKey::Nar(nar_file_name) => {
                self.nars.lock().unwrap().get(nar_file_name).cloned()
            },
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                self.nar_infos.lock().unwrap().get(narinfo_file_name).cloned()
            },
        })
    }

    async fn set<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
        bytes: &[u8],
    ) -> Result<bool, Self::Error> {
        Ok(match key.into() {
            ObjectStorageKey::Nar(nar_file_name) => self
                .nars
                .lock()
                .unwrap()
                .insert(nar_file_name.clone(), bytes.to_owned().into())
                .is_none(),

            ObjectStorageKey::NarInfo(narinfo_file_name) => self
                .nar_infos
                .lock()
                .unwrap()
                .insert(narinfo_file_name.clone(), bytes.to_owned().into())
                .is_none(),
        })
    }
}
