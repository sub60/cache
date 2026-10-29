use core::fmt;
use core::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::http;
use nix_types::{CacheSigningKey, KeyOrSignatureName};
use sub60_cache::{CacheDb, CacheId, CacheInfos, CacheName, UserName};

/// TODO: docs.
#[derive(Clone, Default)]
pub struct InMemoryCacheDb {
    inner: Arc<Mutex<InMemoryCacheDbInner>>,
}

#[derive(Default)]
struct InMemoryCacheDbInner {
    users: HashSet<UserName>,
    caches: HashMap<(UserName, CacheName), CacheInfos>,
}

pub enum Never {}

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

    /// Adds a cache to the DB and returns its infos.
    ///
    /// Every cache gets its own signing key, named `{user}/{name}-1`.
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
    ) -> CacheInfos {
        let user = user.try_into().expect("invalid user name");
        let name = name.try_into().expect("invalid cache name");

        self.with_inner(|inner| {
            if !inner.users.contains(&user) {
                panic!("user does not exist in cache database");
            }

            let key_name = format!("{}/{}-1", &*user, &*name);

            let user_and_name = (user, name);
            if inner.caches.contains_key(&user_and_name) {
                panic!("cache already exists in cache database");
            }

            let seed = inner.caches.len() as u8;

            let cache_infos = CacheInfos {
                id: CacheId::new(Duration::ZERO, [seed; 10]),
                priority: 40,
                signing_key: dummy_signing_key(&key_name, seed),
                store_dir: "/nix/store".into(),
            };

            inner.caches.insert(user_and_name, cache_infos.clone());

            cache_infos
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
    type Error = Never;

    async fn get_infos(
        &self,
        user: UserName,
        cache: CacheName,
    ) -> Result<Option<CacheInfos>, Self::Error> {
        Ok(self.with_inner(|inner| inner.caches.get(&(user, cache)).cloned()))
    }
}

fn dummy_signing_key(name: &str, seed: u8) -> CacheSigningKey {
    CacheSigningKey::new(
        name.parse::<KeyOrSignatureName>().expect("valid name"),
        [seed; 32],
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
