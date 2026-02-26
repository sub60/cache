use core::fmt;
use core::str::FromStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ObjectStorage;
use crate::types::{NarFileName, NixStoreHash};

/// An [`ObjectStorage`] trait that stores the object in memory using a
/// `HashMap`.
#[derive(Clone, Default)]
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
    fn from(nar_file_name: &NarFileName) -> Self {
        Self(format!("nar-{nar_file_name}"))
    }
}

impl From<&NixStoreHash> for InMemoryObjectStorageKey {
    fn from(store_hash: &NixStoreHash) -> Self {
        Self(format!("narinfo-{store_hash}"))
    }
}

impl fmt::Display for InMemoryObjectStorageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for InMemoryObjectStorageKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key_kind, _key) = s
            .split_once('-')
            .ok_or_else(|| format!("invalid key format: {s}"))?;

        match key_kind {
            "nar" | "narinfo" => Ok(Self(s.to_owned())),
            _ => Err(format!("invalid key kind: {key_kind}")),
        }
    }
}
