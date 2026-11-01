use core::fmt;
use core::time::Duration;

use axum::http;

use crate::{CacheInfos, CacheName, CiJob, NamespaceName, TokenHash};

/// TODO: docs.
pub trait CacheDb {
    /// TODO: docs.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// Returns the infos of the cache called `cache` in the namespace called
    /// `namespace`, or `None` if there isn't one.
    ///
    /// Both must be looked up by name the same way [`NamespaceName`]'s and
    /// [`CacheName`]'s `PartialEq` impls compare names, i.e. ignoring ASCII
    /// case.
    fn get_infos(
        &self,
        namespace: NamespaceName,
        cache: CacheName,
    ) -> impl Future<Output = Result<Option<CacheInfos>, Self::Error>> + Send;

    /// Stores the hash of a token that authenticates the given CI job until
    /// `expires_at`, the time since the UNIX epoch it stops being valid at.
    ///
    /// Implementations must only store it if the job can access at least one
    /// cache, and return whether they did.
    fn insert_ci_job_token(
        &self,
        token_hash: TokenHash,
        job: &CiJob,
        expires_at: Duration,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}
