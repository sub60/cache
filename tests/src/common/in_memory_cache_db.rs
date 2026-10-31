use core::fmt;
use core::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::http;
use nix_types::{CacheSigningKey, KeyOrSignatureName};
use sub60_cache::{CacheDb, CacheId, CacheInfos, CacheName, NamespaceName};

/// TODO: docs.
#[derive(Clone, Default)]
pub struct InMemoryCacheDb {
    inner: Arc<Mutex<InMemoryCacheDbInner>>,
}

#[derive(Default)]
struct InMemoryCacheDbInner {
    namespaces: HashSet<NamespaceName>,
    caches: HashMap<(NamespaceName, CacheName), CacheEntry>,
    /// The number of signing keys created so far, used to seed each new key.
    num_signing_keys: u8,
}

struct CacheEntry {
    infos: CacheInfos,
    /// The version of the cache's current signing key, starting at 1 and
    /// incremented on every rotation.
    signing_key_version: u8,
}

pub enum Never {}

impl InMemoryCacheDb {
    /// Adds a namespace to the DB.
    ///
    /// # Panics
    ///
    /// Panics if the DB already contains a namespace with the same name.
    #[track_caller]
    pub(crate) fn add_namespace(
        &self,
        name: impl TryInto<NamespaceName, Error: fmt::Debug>,
    ) {
        let name = name.try_into().expect("invalid namespace name");

        self.with_inner(|inner| {
            if !inner.namespaces.insert(name) {
                panic!("namespace already exists in cache database");
            }
        });
    }

    /// Adds a cache to the DB and returns its infos.
    ///
    /// Every cache gets its own signing key, named `{namespace}/{name}-1`.
    ///
    /// # Panics
    ///
    /// Panics if the namespace does not exist, or if the namespace already has
    /// a cache with the same name.
    #[track_caller]
    pub(crate) fn add_cache(
        &mut self,
        namespace: impl TryInto<NamespaceName, Error: fmt::Debug>,
        name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CacheInfos {
        let namespace = namespace.try_into().expect("invalid namespace name");
        let name = name.try_into().expect("invalid cache name");

        self.with_inner(|inner| {
            if !inner.namespaces.contains(&namespace) {
                panic!("namespace does not exist in cache database");
            }

            let namespace_and_name = (namespace, name);
            if inner.caches.contains_key(&namespace_and_name) {
                panic!("cache already exists in cache database");
            }

            let seed = inner.caches.len() as u8;
            let signing_key_version = 1;

            let cache_infos = CacheInfos {
                id: CacheId::new(Duration::ZERO, [seed; 10]),
                priority: 40,
                signing_key: inner
                    .new_signing_key(&namespace_and_name, signing_key_version),
                store_dir: "/nix/store".into(),
            };

            inner.caches.insert(
                namespace_and_name,
                CacheEntry { infos: cache_infos.clone(), signing_key_version },
            );

            cache_infos
        })
    }

    /// Replaces the signing key of the given cache with a new one, named
    /// `{namespace}/{name}-{version}` where `version` is one more than the
    /// previous key's, and returns the cache's updated infos.
    ///
    /// # Panics
    ///
    /// Panics if the cache does not exist.
    #[track_caller]
    pub(crate) fn rotate_signing_key(
        &mut self,
        namespace: impl TryInto<NamespaceName, Error: fmt::Debug>,
        name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CacheInfos {
        let namespace = namespace.try_into().expect("invalid namespace name");
        let name = name.try_into().expect("invalid cache name");
        let namespace_and_name = (namespace, name);

        self.with_inner(|inner| {
            let signing_key_version = inner
                .caches
                .get(&namespace_and_name)
                .expect("cache does not exist in cache database")
                .signing_key_version
                + 1;

            let signing_key =
                inner.new_signing_key(&namespace_and_name, signing_key_version);

            let entry = inner
                .caches
                .get_mut(&namespace_and_name)
                .expect("checked above");

            entry.infos.signing_key = signing_key;
            entry.signing_key_version = signing_key_version;

            entry.infos.clone()
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
        namespace: NamespaceName,
        cache: CacheName,
    ) -> Result<Option<CacheInfos>, Self::Error> {
        Ok(self.with_inner(|inner| {
            inner
                .caches
                .get(&(namespace, cache))
                .map(|entry| entry.infos.clone())
        }))
    }
}

impl InMemoryCacheDbInner {
    /// Creates a new signing key for the given cache, named
    /// `{namespace}/{name}-{version}`, with a seed that's unique within the DB.
    fn new_signing_key(
        &mut self,
        (namespace, name): &(NamespaceName, CacheName),
        version: u8,
    ) -> CacheSigningKey {
        let seed = self.num_signing_keys;
        self.num_signing_keys += 1;

        CacheSigningKey::new(
            format!("{}/{}-{version}", &**namespace, &**name)
                .parse::<KeyOrSignatureName>()
                .expect("valid name"),
            [seed; 32],
        )
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
