//! TODO: docs.

use core::str::FromStr;
use core::time::Duration;
use core::{fmt, str};

use axum::Extension;
use axum::body::Body;
use axum::http::{self, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use compact_str::{CompactString, format_compact};
use nix_types::{NarFileName, NarInfo, NarInfoFromStrError};

use crate::api::endpoints::narinfo_get;
use crate::context::ObjectStorageError;
use crate::object_gateway::{ObjectGateway, ObjectUrlRequest};
use crate::{
    CacheInfos,
    Cdn,
    Context,
    ObjectStorage,
    ObjectStorageKey,
    endpoints,
};

const NARINFO_CONTENT_TYPE: HeaderValue =
    HeaderValue::from_static("text/x-nix-narinfo");

/// The response returned to GET requests to the NARInfo route, along with the
/// headers telling the CDN how long to cache it for.
struct NarInfoGetHttpResponse<Cdn: crate::Cdn> {
    response: narinfo_get::Response,
    cache_headers: Option<Cdn::CacheHeaders>,
}

/// The type of error that can occur when handling a GET request to the NARInfo
/// route.
#[derive(
    Debug, PartialEq, Eq, derive_more::Display, cauchy::Error, cauchy::From,
)]
enum NarInfoGetResponseError<ObjectStorageError> {
    /// The NARInfo couldn't be retrieved from the object storage.
    GetNarInfo(ObjectStorageError),

    /// The NARInfo bytes retrieved from the object storage were not valid
    /// UTF-8. This should basically never happen unless there's some kind of
    /// data corruption in the object storage or in the transport between the
    /// object storage and this service.
    NarInfoNotUtf8(str::Utf8Error),

    /// The NARInfo string retrieved from the object storage did not parse back
    /// into a valid [`NarInfo`]. Like the [`NarInfoNotUtf8`] error, this should
    /// also never happen unless the data is corrupted in some way.
    ParseNarInfo(#[from] NarInfoFromStrError<<EmptyString as FromStr>::Err>),
}

struct EmptyString;

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    endpoints::method_router::<narinfo_get::Endpoint, _, _>(
        async move |path, Extension(cache_infos)| {
            handler(path, cache_infos, &mut ctx).await
        },
    )
}

async fn handler<Ctx: Context>(
    path: narinfo_get::Path,
    cache_infos: CacheInfos,
    ctx: &mut Ctx,
) -> Result<
    NarInfoGetHttpResponse<Ctx::Cdn>,
    NarInfoGetResponseError<ObjectStorageError<Ctx>>,
> {
    let object_storage = ctx.object_storage();

    let Some(narinfo_bytes) = object_storage
        .get(ObjectStorageKey::NarInfo(
            &cache_infos.id,
            &path.narinfo_file_name,
        ))
        .await
        .map_err(NarInfoGetResponseError::GetNarInfo)?
    else {
        return Ok(NarInfoGetHttpResponse {
            response: narinfo_get::Response::NotFound,
            cache_headers: None,
        });
    };

    let mut narinfo_without_url = str::from_utf8(narinfo_bytes.as_ref())
        .map_err(NarInfoGetResponseError::NarInfoNotUtf8)?
        .parse::<NarInfo<EmptyString>>()?;

    // NARInfos are stored unsigned, so we sign them on the fly with the cache's
    // current signing key.
    let cache_signature = cache_infos
        .signing_key
        .sign(narinfo_without_url.fingerprint().to_string().as_bytes());

    narinfo_without_url.signatures.push(cache_signature);

    let nar_filename = NarFileName::from(&narinfo_without_url);

    let nar_url = ctx.object_gateway().object_url(ObjectUrlRequest {
        key: ObjectStorageKey::Nar(&nar_filename),
        method: http::Method::GET,
        min_ttl: Duration::from_hours(1),
    });

    Ok(NarInfoGetHttpResponse {
        response: narinfo_get::Response::Found(
            narinfo_without_url.with_url(nar_url.as_ref().to_owned()),
        ),
        // The NAR is purely content-addressed, so we can cache it for a long
        // time.
        cache_headers: Some(ctx.cdn().cache_headers(Duration::from_days(365))),
    })
}

impl<ObjectStorageError> NarInfoGetResponseError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::GetNarInfo(_)
            | Self::NarInfoNotUtf8(_)
            | Self::ParseNarInfo(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<Cdn: crate::Cdn> IntoResponse for NarInfoGetHttpResponse<Cdn> {
    fn into_response(self) -> axum::response::Response {
        let mut builder = axum::response::Response::builder()
            .status(self.response.status_code());

        for (name, value) in self.cache_headers.into_iter().flatten() {
            builder = builder.header(name, value);
        }

        let body = match self.response {
            narinfo_get::Response::Found(narinfo) => {
                builder =
                    builder.header(header::CONTENT_TYPE, NARINFO_CONTENT_TYPE);
                Body::new(narinfo.to_string())
            },
            narinfo_get::Response::NotFound => Body::empty(),
        };

        builder.body(body).expect("headers and body are valid")
    }
}

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarInfoGetResponseError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}

impl FromStr for EmptyString {
    type Err = CompactString;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self)
        } else {
            Err(format_compact!("expected empty string, found {s:?}"))
        }
    }
}
