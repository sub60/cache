use core::{fmt, str};

use axum::http;
use cache::{CacheDb, CacheInfos};
use nix_types::CacheSigningKey;
use nix_types::sub60::{CacheName, UserName};

#[derive(Clone)]
pub(crate) struct D1CacheDatabase {
    inner: sqlx_d1::D1Connection,
}

#[derive(Debug)]
pub(crate) enum D1CacheDbError {
    Query(sqlx_d1::Error),
    InvalidPriority(i64),
    InvalidSigningKey(<CacheSigningKey as str::FromStr>::Err),
}

impl D1CacheDatabase {
    pub(crate) fn new(env: &worker::Env) -> Self {
        Self {
            inner: sqlx_d1::D1Connection::new(
                env.d1(env!("D1_DB_BINDING_NAME"))
                    .expect("D1 database binding not found"),
            ),
        }
    }
}

impl CacheDb for D1CacheDatabase {
    type CacheId = i64;
    type Error = D1CacheDbError;

    async fn get_infos(
        &self,
        user: UserName,
        cache: CacheName,
    ) -> Result<Option<CacheInfos<Self::CacheId>>, Self::Error> {
        let Some(row) = sqlx_d1::query!(
            r#"
            SELECT
                c.id AS "id!: i64",
                c.priority AS "priority!: i64",
                c.signing_key AS "signing_key!: String",
                c.store_dir AS "store_dir!: String"
            FROM caches AS c
            INNER JOIN users AS u ON u.id = c.owner_user_id
            WHERE u.username = ? AND c.name = ?
            LIMIT 1
            "#,
            &*user,
            &*cache,
        )
        .fetch_optional(&self.inner)
        .await
        .map_err(D1CacheDbError::Query)?
        else {
            return Ok(None);
        };

        let priority = u32::try_from(row.priority)
            .map_err(|_| D1CacheDbError::InvalidPriority(row.priority))?;

        let signing_key = row
            .signing_key
            .parse()
            .map_err(D1CacheDbError::InvalidSigningKey)?;

        Ok(Some(CacheInfos {
            id: row.id,
            priority,
            signing_key,
            store_dir: row.store_dir.into(),
        }))
    }
}

impl fmt::Display for D1CacheDbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query(err) => write!(f, "D1 query failed: {err}"),
            Self::InvalidPriority(priority) => {
                write!(f, "cache priority {priority} does not fit in u32")
            },
            Self::InvalidSigningKey(err) => {
                write!(f, "cache signing key is invalid: {err}")
            },
        }
    }
}

impl From<D1CacheDbError> for http::StatusCode {
    fn from(err: D1CacheDbError) -> Self {
        match err {
            D1CacheDbError::Query(_)
            | D1CacheDbError::InvalidPriority(_)
            | D1CacheDbError::InvalidSigningKey(_) => {
                Self::INTERNAL_SERVER_ERROR
            },
        }
    }
}
