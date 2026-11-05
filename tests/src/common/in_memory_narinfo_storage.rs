use core::convert::Infallible;
use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use nix_types::{NarInfo, NarInfoFileName, StoreHash};
use sub60_cache::{CacheId, CacheInfos, NarInfoStorage};

/// A [`NarInfoStorage`] that stores NARInfos in memory.
#[derive(Clone, Default)]
pub struct InMemoryNarInfoStorage(
    Arc<Mutex<HashMap<(CacheId, NarInfoFileName), Bytes>>>,
);

impl InMemoryNarInfoStorage {
    fn with_narinfos<R>(
        &self,
        f: impl FnOnce(&mut HashMap<(CacheId, NarInfoFileName), Bytes>) -> R,
    ) -> R {
        #[expect(clippy::unwrap_used)]
        let mut narinfos = self.0.lock().unwrap();
        f(&mut narinfos)
    }
}

impl NarInfoStorage for InMemoryNarInfoStorage {
    type Error = Infallible;

    async fn contains(
        &self,
        cache: &CacheInfos,
        narinfo: &NarInfoFileName,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_narinfos(|narinfos| {
            narinfos.contains_key(&(cache.id, narinfo.clone()))
        }))
    }

    async fn get(
        &self,
        cache: &CacheInfos,
        narinfo: &NarInfoFileName,
    ) -> Result<Option<Bytes>, Self::Error> {
        Ok(self.with_narinfos(|narinfos| {
            narinfos.get(&(cache.id, narinfo.clone())).cloned()
        }))
    }

    async fn insert(
        &self,
        cache: &CacheInfos,
        file_name: &NarInfoFileName,
        narinfo: &NarInfo<impl fmt::Display + Sync>,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_narinfos(|narinfos| {
            let key = (cache.id, file_name.clone());
            if narinfos.contains_key(&key) {
                return false;
            }
            narinfos.insert(key, narinfo.to_string().into());
            true
        }))
    }

    async fn missing(
        &self,
        cache: &CacheInfos,
        store_hashes: &[StoreHash],
    ) -> Result<Vec<StoreHash>, Self::Error> {
        Ok(self.with_narinfos(|narinfos| {
            store_hashes
                .iter()
                .copied()
                .filter(|&store_hash| {
                    let file_name = NarInfoFileName { store_hash };
                    !narinfos.contains_key(&(cache.id, file_name))
                })
                .collect()
        }))
    }
}
