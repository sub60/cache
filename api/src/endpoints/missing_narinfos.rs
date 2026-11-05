//! The `POST /{namespace}/{cache}/missing-narinfos` endpoint.

use core::str::FromStr;
use core::{fmt, ops};

use http::Method;
use nix_types::{StoreHash, StoreHashFromStrError};

use crate::{CachePath, PathParams};

/// The maximum number of store hashes in a request.
pub const MAX_STORE_HASHES: usize = 10_000;

/// The store hashes of the NARInfos to look up, at most [`MAX_STORE_HASHES`] of
/// them.
pub type RequestBody = StoreHashes;

/// The store hashes of the requested NARInfos the cache doesn't have, in the
/// order they were requested.
pub type ResponseBody = StoreHashes;

/// Returns which of the given NARInfos the cache doesn't have.
///
/// It answers what a `HEAD` request for each of the NARInfos would, in a single
/// request.
#[derive(Debug, Copy, Clone)]
pub struct Endpoint;

/// The path parameters of the endpoint's requests.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::AsRef, PathParams)]
#[route(CachePath + "/missing-narinfos")]
pub struct Path {
    /// The cache the NARInfos are in.
    #[as_ref]
    pub cache: CachePath,
}

/// A list of store hashes, written one per line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoreHashes(pub Vec<StoreHash>);

/// The type of error that can occur when parsing a string into
/// [`StoreHashes`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
#[display("line {line}: {error}")]
pub struct StoreHashesFromStrError {
    /// The line with the invalid store hash, starting from 1.
    pub line: usize,

    /// Why the line isn't a valid store hash.
    pub error: StoreHashFromStrError,
}

impl crate::Endpoint for Endpoint {
    const METHOD: Method = Method::POST;

    type Path = Path;
    type Request = RequestBody;
    type Response = ResponseBody;
}

impl fmt::Display for StoreHashes {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|store_hash| writeln!(f, "{}", store_hash.as_str()))
    }
}

impl ops::Deref for StoreHashes {
    type Target = [StoreHash];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for StoreHashes {
    type Err = StoreHashesFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .enumerate()
            .map(|(idx, line)| {
                line.parse().map_err(|error| StoreHashesFromStrError {
                    line: idx + 1,
                    error,
                })
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for StoreHashes {
    fn into_response(self) -> axum::response::Response {
        use axum::http::{HeaderValue, header};

        const CONTENT_TYPE: HeaderValue =
            HeaderValue::from_static("text/plain; charset=utf-8");

        ([(header::CONTENT_TYPE, CONTENT_TYPE)], self.to_string())
            .into_response()
    }
}
