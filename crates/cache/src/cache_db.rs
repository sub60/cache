use core::fmt;

use axum::http;
use types::{CacheName, UserName};

use crate::CacheInfos;

/// TODO: docs.
pub trait CacheDb {
    /// TODO: docs.
    type CacheId: Clone + Send + Sync + 'static;

    /// TODO: docs.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// TODO: docs.
    fn get_infos(
        &self,
        user: UserName,
        cache: CacheName,
    ) -> impl Future<
        Output = Result<Option<CacheInfos<Self::CacheId>>, Self::Error>,
    > + Send;
}
