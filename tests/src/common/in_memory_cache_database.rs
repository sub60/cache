use core::fmt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::http;
use cache::{CacheDb, CacheInfos};
use nix_types::{CacheName, UserName};

type CacheId = (UserName, CacheName);

#[derive(Clone, Default)]
pub(crate) struct InMemoryCacheDb {
    inner: Arc<Mutex<InMemoryCacheDbInner>>,
}

#[derive(Default)]
struct InMemoryCacheDbInner {
    map: HashMap<CacheId, CacheInfos<CacheId>>,
}

pub(crate) enum Never {}

impl InMemoryCacheDb {
    /// Inserts the given cache infos into the DB.
    ///
    /// # Panics
    ///
    /// Panics if the DB already contains another entry with the same
    /// `(user_name, cache_name)` pair.
    #[track_caller]
    #[expect(unused)]
    pub(crate) fn insert(&self, infos: CacheInfos<CacheId>) {
        self.with_map(|map| {
            if let Some(infos) = map.insert(infos.id.clone(), infos) {
                panic!(
                    "an entry with ID {:?} already exists in the cache \
                     database",
                    infos.id
                )
            }
        })
    }

    fn with_map<R>(
        &self,
        f: impl FnOnce(&mut HashMap<CacheId, CacheInfos<CacheId>>) -> R,
    ) -> R {
        f(&mut self.inner.lock().expect("mutex poisoned").map)
    }
}

impl CacheDb for InMemoryCacheDb {
    type CacheId = CacheId;
    type Error = Never;

    async fn get_infos(
        &self,
        user: UserName,
        cache: CacheName,
    ) -> Result<Option<CacheInfos<Self::CacheId>>, Self::Error> {
        Ok(self.with_map(|map| map.get(&(user, cache)).cloned()))
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl From<Never> for http::StatusCode {
    fn from(value: Never) -> Self {
        match value {}
    }
}
