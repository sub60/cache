//! TODO: docs.

use core::num::NonZeroU16;
use core::time::Duration;
use core::{array, fmt};

use axum::Extension;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use nix_types::NarFileName;

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
    FromStrBody(nar_file_name): FromStrBody<NarFileName>,
    ctx: &mut Ctx,
) -> Result<nar_multiparts::Response, NarMultipartsError<NarStorageError<Ctx>>>
{
    let nar_storage = ctx.nar_storage();

    // NARs are content-addressed, so if one already exists with this file
    // name there's no need to upload it again.
    if nar_storage
        .contains(&cache_infos.id, &nar_file_name)
        .await
        .map_err(NarMultipartsError::CheckNarExistence)?
    {
        return Ok(nar_multiparts::Response::AlreadyExists);
    }

    let upload_id = nar_storage
        .create_multipart_upload(&cache_infos.id, &nar_file_name)
        .await
        .map_err(NarMultipartsError::CreateMultipartUpload)?;

    let now = ctx.clock().unix_ts();

    let min_ttl = Duration::from_mins(10);

    let parts_urls = array::from_fn(|part_idx| {
        nar_storage
            .multipart_upload_part_url(
                MultipartUpload {
                    cache_id: &cache_infos.id,
                    nar: &nar_file_name,
                    id: &upload_id,
                },
                NonZeroU16::MIN.saturating_add(part_idx as u16),
                min_ttl,
            )
            .as_ref()
            .parse()
            .expect("valid URL")
    });

    Ok(nar_multiparts::Response::Created(nar_multiparts::ResponseBody {
        upload_id: upload_id.to_string(),
        part_size: nar_storage.part_size(),
        parts_urls,
        expiration_ts: now + min_ttl,
    }))
}

impl<NarStorageError> NarMultipartsError<NarStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::CheckNarExistence(_) | Self::CreateMultipartUpload(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
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

impl<NarStorageError: fmt::Display> IntoResponse
    for NarMultipartsError<NarStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
