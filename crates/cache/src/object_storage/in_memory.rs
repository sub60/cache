use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use nix_types::{NarFileName, NarInfoFileName};

use crate::ObjectStorage;
use crate::object_storage::ObjectStorageKey;

/// An [`ObjectStorage`] trait that stores the object in memory using a
/// `HashMap`.
#[derive(Clone, Default)]
pub struct InMemoryObjectStorage(Arc<Mutex<InMemoryObjectStorageInner>>);

#[derive(Default)]
struct InMemoryObjectStorageInner {
    nars: HashMap<NarFileName, Bytes>,
    nar_infos: HashMap<NarInfoFileName, Bytes>,
}

impl InMemoryObjectStorage {
    fn with_inner<R>(
        &self,
        f: impl FnOnce(&mut InMemoryObjectStorageInner) -> R,
    ) -> R {
        #[expect(clippy::unwrap_used)]
        let mut inner = self.0.lock().unwrap();
        f(&mut inner)
    }
}

impl ObjectStorage for InMemoryObjectStorage {
    type Error = core::convert::Infallible;

    async fn contains_key<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_inner(|inner| match key.into() {
            ObjectStorageKey::Nar(nar_file_name) => {
                inner.nars.contains_key(nar_file_name)
            },
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                inner.nar_infos.contains_key(narinfo_file_name)
            },
        }))
    }

    async fn get<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
    ) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.with_inner(|inner| match key.into() {
            ObjectStorageKey::Nar(nar_file_name) => {
                inner.nars.get(nar_file_name).cloned()
            },
            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                inner.nar_infos.get(narinfo_file_name).cloned()
            },
        }))
    }

    async fn set<'a>(
        &self,
        key: impl Into<ObjectStorageKey<'a>>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<(), Self::Error> {
        self.with_inner(|inner| match key.into() {
            ObjectStorageKey::Nar(nar_file_name) => inner.nars.insert(
                nar_file_name.clone(),
                bytes.as_ref().to_owned().into(),
            ),

            ObjectStorageKey::NarInfo(narinfo_file_name) => {
                inner.nar_infos.insert(
                    narinfo_file_name.clone(),
                    bytes.as_ref().to_owned().into(),
                )
            },
        });

        Ok(())
    }
}
