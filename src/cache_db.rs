use core::fmt;

use axum::http;

use crate::{CacheInfos, CacheName, NamespaceName};

/// TODO: docs.
pub trait CacheDb {
    /// TODO: docs.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// Returns the infos of the cache called `cache` in the namespace called
    /// `namespace`, or `None` if there isn't one.
    ///
    /// The namespace must be looked up by name the same way
    /// [`NamespaceName`]'s `PartialEq` impl compares names, i.e. ignoring
    /// ASCII case.
    fn get_infos(
        &self,
        namespace: NamespaceName,
        cache: CacheName,
    ) -> impl Future<Output = Result<Option<CacheInfos>, Self::Error>> + Send;
}
