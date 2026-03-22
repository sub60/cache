//! TODO: docs.

use core::str::FromStr;
use core::time::Duration;
use core::{fmt, str};

use axum::body::Body;
use axum::extract::Path;
use axum::http::{self, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use compact_str::{CompactString, format_compact};
use types::{NarFileName, NarInfo, NarInfoFileName, NarInfoFromStrError};

use crate::context::ObjectStorageError;
use crate::object_gateway::{ObjectGateway, StorageAccessRequest};
use crate::{Cdn, Context, ObjectStorage};

pub(crate) const ROUTE: &str = "/{narinfo_file_name}";

const NARINFO_CONTENT_TYPE: HeaderValue =
    HeaderValue::from_static("text/x-nix-narinfo");

#[derive(serde::Deserialize)]
struct NarInfoGetPath {
    narinfo_file_name: NarInfoFileName,
}

/// The response returned to GET requests to the NARInfo route.
struct NarInfoGetResponse<Cdn: crate::Cdn> {
    narinfo: String,
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
    #[display("NARInfo at '{_0}' not found")]
    NarInfoNotFound(NarInfoFileName),

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
    routing::get(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(NarInfoGetPath { narinfo_file_name }): Path<NarInfoGetPath>,
    ctx: &mut Ctx,
) -> Result<
    NarInfoGetResponse<Ctx::Cdn>,
    NarInfoGetResponseError<ObjectStorageError<Ctx>>,
> {
    let object_storage = ctx.object_storage();

    let Some(narinfo_bytes) = object_storage
        .get(&narinfo_file_name)
        .await
        .map_err(NarInfoGetResponseError::GetNarInfo)?
    else {
        return Err(NarInfoGetResponseError::NarInfoNotFound(
            narinfo_file_name,
        ));
    };

    let narinfo_without_url = str::from_utf8(narinfo_bytes.as_ref())
        .map_err(NarInfoGetResponseError::NarInfoNotUtf8)?
        .parse::<NarInfo<EmptyString>>()?;

    let nar_filename = NarFileName::from(&narinfo_without_url);

    let nar_url = ctx.object_gateway().object_url(StorageAccessRequest {
        key: (&nar_filename).into(),
        method: http::Method::GET,
        min_ttl: Duration::from_hours(1),
    });

    Ok(NarInfoGetResponse {
        narinfo: narinfo_without_url.with_url(nar_url.as_ref()).to_string(),
        // The NAR is purely content-addressed, so we can cache it for a long
        // time.
        cache_headers: ctx.cdn().cache_headers(Duration::from_days(365)),
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
            .header(header::CONTENT_TYPE, NARINFO_CONTENT_TYPE);

        for (name, value) in self.cache_headers.into_iter() {
            builder = builder.header(name, value);
        }

        builder
            .body(Body::new(self.narinfo))
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
