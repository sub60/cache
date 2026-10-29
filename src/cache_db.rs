use core::fmt;

use axum::http;

use crate::{CacheInfos, CacheName, UserName};

/// TODO: docs.
pub trait CacheDb {
    /// TODO: docs.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// TODO: docs.
    fn get_infos(
        &self,
        user: UserName,
        cache: CacheName,
    ) -> impl Future<Output = Result<Option<CacheInfos>, Self::Error>> + Send;
}
