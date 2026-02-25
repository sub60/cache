use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ObjectStorage;
use crate::types::{NarFileName, Nix32Digest};

/// An [`ObjectStorage`] trait that stores the object in memory using a
/// `HashMap`.
pub struct InMemoryObjectStorage {
    objects: Arc<Mutex<HashMap<InMemoryObjectStorageKey, Vec<u8>>>>,
}

/// The key of the [`InMemoryObjectStorage`].
#[derive(PartialEq, Eq, Hash)]
pub struct InMemoryObjectStorageKey(String);

#[expect(clippy::unwrap_used)]
impl ObjectStorage for InMemoryObjectStorage {
    type Key<'a> = InMemoryObjectStorageKey;
    type Error = core::convert::Infallible;

    async fn contains_key(
        &self,
        key: &Self::Key<'_>,
    ) -> Result<bool, Self::Error> {
        Ok(self.objects.lock().unwrap().contains_key(key))
    }

    async fn get(
        &self,
        key: &Self::Key<'_>,
    ) -> Result<Option<impl AsRef<[u8]> + use<>>, Self::Error> {
        Ok(self.objects.lock().unwrap().get(key).cloned())
    }

    async fn set(
        &self,
        key: Self::Key<'_>,
        bytes: &[u8],
    ) -> Result<bool, Self::Error> {
        Ok(self.objects.lock().unwrap().insert(key, bytes.to_vec()).is_none())
    }
}

impl From<&NarFileName> for InMemoryObjectStorageKey {
    fn from(value: &NarFileName) -> Self {
        Self(value.to_string())
    }
}

impl From<&Nix32Digest<20>> for InMemoryObjectStorageKey {
    fn from(value: &Nix32Digest<20>) -> Self {
        Self(value.to_string())
    }
}
