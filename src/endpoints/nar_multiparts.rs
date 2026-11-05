//! TODO: docs.

use core::fmt;
use core::num::{NonZeroU16, NonZeroU64};
use core::time::Duration;

use axum::Extension;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use smallvec::SmallVec;

use crate::api::endpoints::nar_multiparts;
use crate::context::NarStorageError;
use crate::from_str_body::FromStrBody;
use crate::{
    CacheInfos,
    Clock,
    Context,
    MultipartUpload,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarMultipartsError<NarStorageError> {
    CheckNarExistence(NarStorageError),
    CreateMultipartUpload(NarStorageError),
    GetPartUrls(NarStorageError),
    #[display("NAR of {size} bytes is larger than the maximum of {max}")]
    TooLarge {
        size: NonZeroU64,
        max: u64,
    },
}

impl endpoints::CacheEndpoint for nar_multiparts::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PushToCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |Extension(cache_infos), body| {
                handler(cache_infos, body, &mut ctx).await.map(into_response)
            },
        )
    }
}

async fn handler<Ctx: Context>(
    cache_infos: CacheInfos,
    FromStrBody(body): FromStrBody<nar_multiparts::RequestBody>,
    ctx: &mut Ctx,
) -> Result<nar_multiparts::Response, NarMultipartsError<NarStorageError<Ctx>>>
{
    let nar_storage = ctx.nar_storage();

    if let Some(max_multipart_parts) = nar_storage.max_multipart_parts() {
        let max = max_multipart_parts as u64
            * u64::from(nar_storage.part_size().get());

        if body.file_size.get() > max {
            return Err(NarMultipartsError::TooLarge {
                size: body.file_size,
                max,
            });
        }
    }

    // NARs are content-addressed, so if one already exists with this file
    // name there's no need to upload it again.
    if nar_storage
        .contains(&cache_infos, &body.nar_file_name)
        .await
        .map_err(NarMultipartsError::CheckNarExistence)?
    {
        return Ok(nar_multiparts::Response::AlreadyExists);
    }

    let upload_id = nar_storage
        .create_multipart_upload(
            &cache_infos,
            &body.nar_file_name,
            body.file_size,
        )
        .await
        .map_err(NarMultipartsError::CreateMultipartUpload)?;

    let now = ctx.clock().unix_ts();

    let min_ttl = Duration::from_mins(10);

    let part_size = nar_storage.part_size();

    // The NAR's parts, of which only the first ones get URLs right away.
    let num_parts = body.file_size.get().div_ceil(u64::from(part_size.get()));

    let part_numbers = (1..=num_parts.min(4))
        .map(|part_number| {
            NonZeroU16::new(part_number as u16)
                .expect("part numbers start at 1")
        })
        .collect::<SmallVec<[NonZeroU16; 4]>>();

    let urls = nar_storage
        .multipart_upload_part_urls(
            MultipartUpload {
                cache: &cache_infos,
                nar: &body.nar_file_name,
                id: &upload_id,
            },
            &part_numbers,
            min_ttl,
        )
        .await
        .map_err(NarMultipartsError::GetPartUrls)?;

    let parts_urls = urls
        .iter()
        .map(|url| url.as_ref().parse().expect("valid URL"))
        .collect();

    Ok(nar_multiparts::Response::Created(nar_multiparts::ResponseBody {
        upload_id: upload_id.to_string(),
        part_size,
        parts_urls,
        expiration_ts: now + min_ttl,
    }))
}

impl<NarStorageError: Into<StatusCode>> NarMultipartsError<NarStorageError> {
    fn into_status_code(self) -> StatusCode {
        match self {
            Self::CheckNarExistence(err)
            | Self::CreateMultipartUpload(err)
            | Self::GetPartUrls(err) => err.into(),
            Self::TooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}

fn into_response(
    response: nar_multiparts::Response,
) -> axum::response::Response {
    let status_code = response.status_code();

    match response {
        nar_multiparts::Response::AlreadyExists => {
            (status_code, Body::empty()).into_response()
        },
        nar_multiparts::Response::Created(body) => {
            (status_code, body.to_string()).into_response()
        },
    }
}

impl<NarStorageError> IntoResponse for NarMultipartsError<NarStorageError>
where
    NarStorageError: fmt::Display + Into<StatusCode>,
{
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        (self.into_status_code(), message).into_response()
    }
}
