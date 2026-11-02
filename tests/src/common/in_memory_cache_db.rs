use core::fmt;
use core::time::Duration;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use axum::http;
use nix_types::{CacheSigningKey, KeyOrSignatureName};
use smol_str::SmolStr;
use sub60_cache::api::{CacheName, CachePath, NamespaceName};
use sub60_cache::{
    CacheDb,
    CacheId,
    CacheInfos,
    CiJob,
    CommitSha,
    HostSoftware,
    Permission,
    Permissions,
    Token,
    TokenHash,
    TokenKind,
};

/// TODO: docs.
#[derive(Clone, Default)]
pub struct InMemoryCacheDb {
    inner: Arc<Mutex<InMemoryCacheDbInner>>,
}

#[derive(Default)]
struct InMemoryCacheDbInner {
    namespaces: HashSet<NamespaceName>,
    caches: HashMap<(NamespaceName, CacheName), CacheEntry>,
    ci_job_grants: Vec<((NamespaceName, CacheName), CiJobGrant)>,
    ci_job_tokens: HashMap<TokenHash, CiJobToken>,
    /// The number of signing keys created so far, used to seed each new key.
    num_signing_keys: u8,
}

struct CacheEntry {
    infos: CacheInfos,
    /// The version of the cache's current signing key, starting at 1 and
    /// incremented on every rotation.
    signing_key_version: u8,
}

/// Gives the CI jobs matching all its conditions access to a cache.
#[derive(Clone)]
pub struct CiJobGrant {
    /// What the jobs can do to the cache.
    pub permissions: Permissions,
    /// The host the jobs run on.
    pub host: SmolStr,
    /// The host's ID for the namespace owning the jobs' repository.
    pub host_namespace_id: SmolStr,
    /// Matches any repository if `None`.
    pub host_repo_name: Option<SmolStr>,
    /// Matches any ref if `None`.
    pub git_ref: Option<SmolStr>,
}

/// A stored CI job token.
#[derive(Clone, Debug)]
pub struct CiJobToken {
    /// The job the token authenticates.
    pub job: CiJob,
    /// When the token stops being valid, as the time since the UNIX epoch.
    pub expires_at: Duration,
}

/// Why the DB denied a request access to a cache.
#[derive(Debug, PartialEq, Eq, derive_more::Display)]
pub enum CacheAccessDenied {
    #[display("no such cache")]
    NotFound,
    #[display("missing, unknown or expired token")]
    Unauthorized,
    #[display("the token can't access the cache")]
    Forbidden,
}

/// The error type of mocks that never fail.
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

    /// Gives the CI jobs matching the grant access to the given cache.
    ///
    /// # Panics
    ///
    /// Panics if the cache does not exist.
    #[track_caller]
    pub(crate) fn add_ci_job_grant(
        &self,
        namespace: impl TryInto<NamespaceName, Error: fmt::Debug>,
        name: impl TryInto<CacheName, Error: fmt::Debug>,
        grant: CiJobGrant,
    ) {
        let namespace = namespace.try_into().expect("invalid namespace name");
        let name = name.try_into().expect("invalid cache name");

        self.with_inner(|inner| {
            let namespace_and_name = (namespace, name);
            if !inner.caches.contains_key(&namespace_and_name) {
                panic!("cache does not exist in cache database");
            }
            inner.ci_job_grants.push((namespace_and_name, grant));
        });
    }

    /// Stores a token for a new CI job that can only access the given cache,
    /// and returns it.
    ///
    /// # Panics
    ///
    /// Panics if the cache does not exist.
    #[track_caller]
    pub(crate) fn add_ci_job_token(
        &self,
        namespace: impl TryInto<NamespaceName, Error: fmt::Debug>,
        name: impl TryInto<CacheName, Error: fmt::Debug>,
        expires_at: Duration,
    ) -> Token {
        let num_tokens = self.num_ci_job_tokens();

        // Each job gets its own namespace on the host, so the grant added for
        // it doesn't match any other job.
        let job = CiJob {
            host: "https://github.com".into(),
            host_software: HostSoftware::GitHub,
            host_namespace_id: num_tokens.to_string().into(),
            host_repo_id: "1".into(),
            host_repo_name: "repo".into(),
            git_ref: "refs/heads/main".into(),
            commit_sha: CommitSha::Sha1([0; 20]),
            host_run_id: "1".into(),
            run_attempt: Some(1),
            run_url: "https://github.com/owner/repo/actions/runs/1".into(),
        };

        self.add_ci_job_grant(
            namespace,
            name,
            CiJobGrant {
                permissions: Permissions::new([
                    Permission::PullFromCache,
                    Permission::PushToCache,
                ]),
                host: job.host.clone(),
                host_namespace_id: job.host_namespace_id.clone(),
                host_repo_name: None,
                git_ref: None,
            },
        );

        let mut random = [0; 20];
        random[..8].copy_from_slice(&num_tokens.to_le_bytes());
        let token = Token::new(TokenKind::CiJob, random);

        self.with_inner(|inner| {
            inner
                .ci_job_tokens
                .insert(token.hash(), CiJobToken { job, expires_at });
        });

        token
    }

    /// Returns the number of stored CI job tokens.
    pub(crate) fn num_ci_job_tokens(&self) -> usize {
        self.with_inner(|inner| inner.ci_job_tokens.len())
    }

    /// Returns the CI job token with the given hash, if it was stored.
    pub(crate) fn ci_job_token(&self, hash: &TokenHash) -> Option<CiJobToken> {
        self.with_inner(|inner| inner.ci_job_tokens.get(hash).cloned())
    }

    fn with_inner<R>(
        &self,
        f: impl FnOnce(&mut InMemoryCacheDbInner) -> R,
    ) -> R {
        f(&mut self.inner.lock().expect("mutex poisoned"))
    }
}

impl CacheDb for InMemoryCacheDb {
    type Error = CacheAccessDenied;

    async fn authorize_cache_access(
        &self,
        cache: &CachePath,
        permissions: Permissions,
        token: Option<Token>,
        now: Duration,
    ) -> Result<CacheInfos, Self::Error> {
        // All the caches are public, so anyone can pull from them.
        const PUBLIC: Permissions =
            Permissions::new([Permission::PullFromCache]);

        self.with_inner(|inner| {
            let key = (cache.namespace.clone(), cache.cache.clone());

            let entry =
                inner.caches.get(&key).ok_or(CacheAccessDenied::NotFound)?;

            let token = match token {
                Some(token) => token,
                None if PUBLIC.contains_all(permissions) => {
                    return Ok(entry.infos.clone());
                },
                None => return Err(CacheAccessDenied::Unauthorized),
            };

            let job = match token.kind() {
                TokenKind::CiJob => inner.ci_job_tokens.get(&token.hash()),
                // The tests don't store machine tokens.
                TokenKind::Machine => None,
            }
            .filter(|job_token| job_token.expires_at > now)
            .map(|job_token| &job_token.job)
            .ok_or(CacheAccessDenied::Unauthorized)?;

            let granted = inner
                .ci_job_grants
                .iter()
                .filter(|(granted, grant)| {
                    *granted == key && grant.matches(job)
                })
                .fold(PUBLIC, |granted, (_, grant)| {
                    granted | grant.permissions
                });

            if !granted.contains_all(permissions) {
                return Err(CacheAccessDenied::Forbidden);
            }

            Ok(entry.infos.clone())
        })
    }

    async fn insert_ci_job_token(
        &self,
        token_hash: TokenHash,
        job: &CiJob,
        expires_at: Duration,
    ) -> Result<bool, Self::Error> {
        Ok(self.with_inner(|inner| {
            if !inner.ci_job_grants.iter().any(|(_, grant)| grant.matches(job))
            {
                return false;
            }
            inner.ci_job_tokens.insert(
                token_hash,
                CiJobToken { job: job.clone(), expires_at },
            );
            true
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

impl CiJobGrant {
    fn matches(&self, job: &CiJob) -> bool {
        self.host == job.host
            && self.host_namespace_id == job.host_namespace_id
            && self.host_repo_name.as_ref().is_none_or(|name| {
                name.eq_ignore_ascii_case(&job.host_repo_name)
            })
            && self
                .git_ref
                .as_ref()
                .is_none_or(|git_ref| *git_ref == job.git_ref)
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl From<CacheAccessDenied> for http::StatusCode {
    fn from(denied: CacheAccessDenied) -> Self {
        match denied {
            CacheAccessDenied::NotFound => Self::NOT_FOUND,
            CacheAccessDenied::Unauthorized => Self::UNAUTHORIZED,
            CacheAccessDenied::Forbidden => Self::FORBIDDEN,
        }
    }
}

impl From<Never> for http::StatusCode {
    fn from(value: Never) -> Self {
        match value {}
    }
}
