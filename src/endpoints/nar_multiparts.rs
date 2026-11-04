//! TODO: docs.

use core::num::{NonZeroU16, NonZeroU64};
use core::time::Duration;
use core::{array, fmt};

use axum::Extension;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

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

    let part_numbers: [NonZeroU16; 4] = array::from_fn(|part_idx| {
        NonZeroU16::MIN.saturating_add(part_idx as u16)
    });

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

    let parts_urls =
        array::from_fn(|idx| urls[idx].as_ref().parse().expect("valid URL"));

    Ok(nar_multiparts::Response::Created(nar_multiparts::ResponseBody {
        upload_id: upload_id.to_string(),
        part_size: nar_storage.part_size(),
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
