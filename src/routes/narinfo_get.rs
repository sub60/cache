//! TODO: docs.

use core::str::FromStr;
use core::time::Duration;
use core::{fmt, str};

use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use compact_str::{CompactString, format_compact};

use crate::cache_ctx::CacheCtx;
use crate::routes::NAR_INFO_CONTENT_TYPE;
use crate::types::{NarInfo, NarInfoFromStrError, NixStoreHash};
use crate::{Cdn, Context, ObjectStorage};

pub(crate) const ROUTE: &str = "/{store_hash}.narinfo";

/// The response returned to GET requests to the NARInfo route.
struct NarInfoGetResponse<Cdn: crate::Cdn> {
    nar_info: NarInfo<String>,
    cache_headers: Cdn::CacheHeaders,
}

/// The type of error that can occur when handling a GET request to the NARInfo
/// route.
#[derive(
    Debug, PartialEq, Eq, derive_more::Display, cauchy::Error, cauchy::From,
)]
enum NarInfoGetResponseError<ObjectStorageError> {
    /// The NARInfo couldn't be retrieved from the object storage.
    GetNarInfo(ObjectStorageError),

    /// The NARInfo was not found in the object storage.
    #[display("NARInfo at '{_0}.narinfo' not found")]
    NarInfoNotFound(NixStoreHash),

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
    routing::get(async move |cache_ctx, path| {
        handler(cache_ctx, path, &mut ctx).await
    })
}

async fn handler<Ctx: Context>(
    Extension(cache_ctx): Extension<CacheCtx>,
    Path(store_hash): Path<NixStoreHash>,
    ctx: &mut Ctx,
) -> Result<
    NarInfoGetResponse<Ctx::Cdn>,
    NarInfoGetResponseError<<Ctx::ObjectStorage as ObjectStorage>::Error>,
> {
    // TODO: validate that the user has read permissions for that cache.

    let object_storage = ctx.object_storage();

    let object_storage_key = (&store_hash).into();

    let nar_info_bytes = object_storage
        .get(object_storage_key)
        .await
        .map_err(NarInfoGetResponseError::GetNarInfo)?
        .ok_or(NarInfoGetResponseError::NarInfoNotFound(store_hash))?;

    let nar_info_without_url = str::from_utf8(nar_info_bytes.as_ref())
        .map_err(NarInfoGetResponseError::NarInfoNotUtf8)?
        .parse::<NarInfo<EmptyString>>()?;

    let nar_url = if cache_ctx.visibility.is_public() {
        todo!()
    } else {
        String::from("todo!")
    };

    // We only cache the response if the cache is public. If the cache is
    // private, the NARInfo's URL will be a time-limited pre-signed URL which we
    // don't want the CDN to cache.
    let cache_headers = if cache_ctx.visibility.is_public() {
        // The NAR is purely content-addressed, so we can cache it for a long
        // time.
        ctx.cdn().cache_headers(Duration::from_days(365))
    } else {
        ctx.cdn().cache_headers_no_cache()
    };

    Ok(NarInfoGetResponse {
        nar_info: nar_info_without_url.with_url(nar_url),
        cache_headers,
    })
}

impl<ObjectStorageError> NarInfoGetResponseError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NarInfoNotFound(_) => StatusCode::NOT_FOUND,
            Self::GetNarInfo(_)
            | Self::NarInfoNotUtf8(_)
            | Self::ParseNarInfo(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<Cdn: crate::Cdn> IntoResponse for NarInfoGetResponse<Cdn> {
    fn into_response(self) -> axum::response::Response {
        let mut builder = axum::response::Response::builder()
            .header(header::CONTENT_TYPE, NAR_INFO_CONTENT_TYPE);

        for (name, value) in self.cache_headers.into_iter() {
            builder = builder.header(name, value);
        }

        builder
            .body(Body::new(self.nar_info.to_string()))
            .expect("headers and body are valid")
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
