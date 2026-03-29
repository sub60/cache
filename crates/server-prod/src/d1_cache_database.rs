use core::fmt;
use std::sync::Arc;

use axum::http;
use cache::{CacheDb, CacheInfos};
use types::{CacheName, UserName};

#[derive(Clone)]
pub(crate) struct D1CacheDatabase {
    #[expect(unused)]
    inner: Arc<worker::d1::D1Database>,
}

pub(crate) enum D1CacheDbError {}

impl D1CacheDatabase {
    pub(crate) fn new(env: &worker::Env) -> Self {
        Self {
            inner: env
                .d1(option_env!("D1_DB_BINDING_NAME").unwrap_or("todo"))
                .expect("D1 database binding not found")
                .into(),
        }
    }
}

impl CacheDb for D1CacheDatabase {
    type CacheId = ();
    type Error = D1CacheDbError;

    async fn get_infos(
        &self,
        _user: UserName,
        _cache: CacheName,
    ) -> Result<Option<CacheInfos<Self::CacheId>>, Self::Error> {
        todo!()
    }
}

impl fmt::Display for D1CacheDbError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

impl From<D1CacheDbError> for http::StatusCode {
    fn from(err: D1CacheDbError) -> Self {
        match err {}
    }
}
