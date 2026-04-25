use core::fmt;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::http;
use cache::{CacheDb, CacheInfos};
use nix_types::sub60::{CacheName, UserName};
use nix_types::{CacheSigningKey, KeyOrSignatureName};

type CacheId = (UserName, CacheName);

#[derive(Clone, Default)]
pub(crate) struct InMemoryCacheDb {
    inner: Arc<Mutex<InMemoryCacheDbInner>>,
}

#[derive(Default)]
struct InMemoryCacheDbInner {
    users: HashSet<UserName>,
    caches: HashMap<CacheId, CacheInfos<CacheId>>,
}

pub(crate) enum Never {}

impl InMemoryCacheDb {
    /// Adds a user to the DB.
    ///
    /// # Panics
    ///
    /// Panics if the DB already contains a user with the same name.
    #[track_caller]
    pub(crate) fn add_user(
        &self,
        name: impl TryInto<UserName, Error: fmt::Debug>,
    ) {
        let name = name.try_into().expect("invalid user name");

        self.with_inner(|inner| {
            if !inner.users.insert(name) {
                panic!("user already exists in cache database");
            }
        });
    }

    /// Adds a cache to the DB and returns its ID.
    ///
    /// # Panics
    ///
    /// Panics if the user does not exist, or if the user already has a cache
    /// with the same name.
    #[track_caller]
    pub(crate) fn add_cache(
        &mut self,
        user: impl TryInto<UserName, Error: fmt::Debug>,
        name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CacheId {
        let user = user.try_into().expect("invalid user name");
        let name = name.try_into().expect("invalid cache name");

        self.with_inner(|inner| {
            if !inner.users.contains(&user) {
                panic!("user does not exist in cache database");
            }

            let cache_id = (user, name);
            if inner.caches.contains_key(&cache_id) {
                panic!("cache already exists in cache database");
            }

            inner.caches.insert(
                cache_id.clone(),
                CacheInfos {
                    id: cache_id.clone(),
                    priority: 40,
                    signing_key: dummy_signing_key(),
                    store_dir: "/nix/store".into(),
                },
            );

            cache_id
        })
    }

    fn with_inner<R>(
        &self,
        f: impl FnOnce(&mut InMemoryCacheDbInner) -> R,
    ) -> R {
        f(&mut self.inner.lock().expect("mutex poisoned"))
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
        Ok(self.with_inner(|inner| inner.caches.get(&(user, cache)).cloned()))
    }
}

fn dummy_signing_key() -> CacheSigningKey {
    CacheSigningKey::new(
        "test-cache-1".parse::<KeyOrSignatureName>().expect("valid name"),
        [0; 32],
    )
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
