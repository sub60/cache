//! A layer that only lets requests reach a [`CacheEndpoint`]'s handler if they
//! may access the cache it acts on.

use core::fmt;

use axum::extract::{RawPathParams, Request};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use base64::prelude::*;

use crate::api::{PathParams, PathParamsRejection};
use crate::endpoints::CacheEndpoint;
use crate::route_layer::RouteLayer;
use crate::{CacheDb, Clock, Context, Token, TokenFormat};

pub(crate) fn layer<E: CacheEndpoint>(ctx: impl Context) -> impl RouteLayer {
    middleware::from_fn(move |params, req, next| {
        let ctx = ctx.clone();
        async move { handler::<E, _>(params, req, next, &ctx).await }
    })
}

/// The type of error that can occur when checking a request's access to a
/// cache.
#[derive(Debug, derive_more::Display)]
enum CacheAccessError<CacheDbError> {
    /// The request's `Authorization` header doesn't hold a token.
    #[display("invalid Authorization header")]
    InvalidAuthorization,

    /// The cache database failed to look up the cache, or denied access to it.
    #[display("{_0}")]
    CacheDb(CacheDbError),
}

async fn handler<E: CacheEndpoint, Ctx: Context>(
    params: RawPathParams,
    mut req: Request,
    next: Next,
    ctx: &Ctx,
) -> Response {
    let path = match E::Path::parse(&mut params.iter()) {
        Ok(path) => path,
        Err(err) => return PathParamsRejection::Parse(err).into_response(),
    };

    let token = match token::<Ctx>(req.headers(), ctx.token_format()) {
        Ok(token) => token,
        Err(err) => return err.into_response(),
    };

    let cache_infos = match ctx
        .cache_db()
        .authorize_cache_access(
            path.as_ref(),
            E::PERMISSIONS,
            token,
            ctx.clock().unix_ts(),
        )
        .await
    {
        Ok(infos) => infos,
        Err(err) => {
            return CacheAccessError::CacheDb(err).into_response();
        },
    };

    req.extensions_mut().insert(cache_infos);
    next.run(req).await
}

/// Returns the token the request was sent with, if any.
fn token<Ctx: Context>(
    headers: &HeaderMap,
    token_format: &Ctx::TokenFormat,
) -> Result<Option<Token>, CacheAccessError<<Ctx::CacheDb as CacheDb>::Error>> {
    headers
        .get(header::AUTHORIZATION)
        .map(|value| {
            parse_authorization(value, token_format)
                .ok_or(CacheAccessError::InvalidAuthorization)
        })
        .transpose()
}

/// Parses the token in an `Authorization` header.
///
/// The token is either sent as a bearer token, or as the password of basic
/// credentials, which is how Nix sends the ones in its netrc file.
fn parse_authorization(
    value: &HeaderValue,
    token_format: &impl TokenFormat,
) -> Option<Token> {
    let (scheme, credentials) = value.to_str().ok()?.split_once(' ')?;

    if scheme.eq_ignore_ascii_case("Bearer") {
        token_format.parse(credentials)
    } else if scheme.eq_ignore_ascii_case("Basic") {
        let credentials = BASE64_STANDARD.decode(credentials).ok()?;
        let (_user_id, password) =
            str::from_utf8(&credentials).ok()?.split_once(':')?;
        token_format.parse(password)
    } else {
        None
    }
}

impl<CacheDbError> IntoResponse for CacheAccessError<CacheDbError>
where
    CacheDbError: fmt::Display + Into<StatusCode>,
{
    fn into_response(self) -> Response {
        let message = self.to_string();

        let status = match self {
            Self::InvalidAuthorization => StatusCode::UNAUTHORIZED,
            Self::CacheDb(err) => err.into(),
        };

        let mut response = (status, message).into_response();

        // 401s must say how to authenticate.
        if status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer"),
            );
        }

        response
    }
}
