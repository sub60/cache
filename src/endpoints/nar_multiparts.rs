//! TODO: docs.

use core::num::NonZeroU16;
use core::time::Duration;
use core::{array, fmt};

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use nix_types::NarFileName;

use crate::api::endpoints::nar_multiparts;
use crate::context::ObjectStorageError;
use crate::from_str_body::FromStrBody;
use crate::object_gateway::MultipartUploadUrlRequest;
use crate::object_storage::ObjectStorageKey;
use crate::{
    Clock,
    Context,
    ObjectGateway,
    ObjectStorage,
    Permission,
    endpoints,
};

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarMultipartsError<ObjectStorageError> {
    CheckNarExistence(ObjectStorageError),
    CreateMultipartUpload(ObjectStorageError),
}

impl endpoints::CacheEndpoint for nar_multiparts::Endpoint {
    const PERMISSION: Permission = Permission::Write;

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(async move |body| {
            handler(body, &mut ctx).await.map(into_response)
        })
    }
}

async fn handler<Ctx: Context>(
    FromStrBody(nar_file_name): FromStrBody<NarFileName>,
    ctx: &mut Ctx,
) -> Result<nar_multiparts::Response, NarMultipartsError<ObjectStorageError<Ctx>>>
{
    let storage_key = ObjectStorageKey::Nar(&nar_file_name);

    let object_gateway = ctx.object_gateway();

    // NARs are content-addressed, so if one already exists under this key
    // there's no need to upload it again.
    if object_gateway
        .storage()
        .contains_key(storage_key)
        .await
        .map_err(NarMultipartsError::CheckNarExistence)?
    {
        return Ok(nar_multiparts::Response::AlreadyExists);
    }

    let upload_id = object_gateway
        .storage()
        .create_multipart_upload(storage_key)
        .await
        .map_err(NarMultipartsError::CreateMultipartUpload)?;

    let now = ctx.clock().unix_ts();

    let min_ttl = Duration::from_mins(10);

    let parts_urls = array::from_fn(|part_idx| {
        object_gateway
            .multipart_upload_url(MultipartUploadUrlRequest {
                key: storage_key,
                part_number: NonZeroU16::MIN.saturating_add(part_idx as u16),
                upload_id: &upload_id,
                min_ttl,
            })
            .as_ref()
            .parse()
            .expect("valid URL")
    });

    Ok(nar_multiparts::Response::Created(nar_multiparts::ResponseBody {
        upload_id: upload_id.to_string(),
        part_size: object_gateway.storage().part_size(),
        parts_urls,
        expiration_ts: now + min_ttl,
    }))
}

impl<ObjectStorageError> NarMultipartsError<ObjectStorageError> {
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

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarMultipartsError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
