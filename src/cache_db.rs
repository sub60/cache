use core::fmt;

use axum::http;

use crate::{CacheInfos, CacheName, NamespaceName};

/// TODO: docs.
pub trait CacheDb {
    /// TODO: docs.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// TODO: docs.
    fn get_infos(
        &self,
        namespace: NamespaceName,
        cache: CacheName,
    ) -> impl Future<Output = Result<Option<CacheInfos>, Self::Error>> + Send;
}
