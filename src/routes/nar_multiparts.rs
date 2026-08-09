//! TODO: docs.

use core::num::NonZeroU16;
use core::time::Duration;
use core::{array, fmt};

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};
use nix_types::NarFileName;

use crate::context::ObjectStorageError;
use crate::from_str_body::FromStrBody;
use crate::nar_multiparts::NarMultipartsResponseBody;
use crate::object_gateway::MultipartUploadUrlRequest;
use crate::object_storage::ObjectStorageKey;
use crate::{Clock, Context, ObjectGateway, ObjectStorage};

pub(crate) const ROUTE: &str = "/nar-multiparts";

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarMultipartsError<ObjectStorageError> {
    CreateMultipartUpload(ObjectStorageError),
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::post(async move |body| handler(body, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    FromStrBody(nar_file_name): FromStrBody<NarFileName>,
    ctx: &mut Ctx,
) -> Result<
    impl IntoResponse + use<Ctx>,
    NarMultipartsError<ObjectStorageError<Ctx>>,
> {
    // TODO: validate that the user has write permissions for that cache.

    let storage_key = ObjectStorageKey::Nar(&nar_file_name);

    let object_gateway = ctx.object_gateway();

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

    Ok((
        StatusCode::OK,
        NarMultipartsResponseBody {
            upload_id: upload_id.to_string(),
            part_size: object_gateway.storage().part_size(),
            parts_urls,
            expiration_ts: now + min_ttl,
        }
        .to_string(),
    ))
}

impl<ObjectStorageError> NarMultipartsError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::CreateMultipartUpload(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarMultipartsError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
