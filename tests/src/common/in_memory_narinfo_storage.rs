use core::convert::Infallible;
use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use nix_types::{NarInfo, NarInfoFileName};
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
}
