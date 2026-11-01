use core::fmt;
use core::time::Duration;

use axum::http;

use crate::api::CachePath;
use crate::{CacheInfos, CiJob, Permission, Token, TokenHash};

/// TODO: docs.
pub trait CacheDb {
    /// The error returned when a request fails. Its status code is the one sent
    /// to the client.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// Returns the infos of the cache at the given path, if a request sent
    /// with `token` has `permission` on it.
    #[expect(clippy::too_many_arguments)]
    fn authorize_cache_access(
        &self,
        cache: &CachePath,
        permission: Permission,
        token: Option<Token>,
        now: Duration,
    ) -> impl Future<Output = Result<CacheInfos, Self::Error>> + Send;

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
