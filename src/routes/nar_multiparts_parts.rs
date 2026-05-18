//! TODO: docs.

use core::str;
use core::time::Duration;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::MultipartUploadId;
use crate::from_str_body::FromStrBody;
use crate::nar_multiparts::{PartsRequestBody, PartsResponseBody};
use crate::object_gateway::MultipartUploadUrlRequest;
use crate::object_storage::ObjectStorageKey;
use crate::{Clock, Context, ObjectGateway};

pub(crate) const ROUTE: &str = "/nar-multiparts/{upload_id}/parts";

#[derive(serde::Deserialize)]
struct NarMultipartsPartsPath<UploadId> {
    upload_id: UploadId,
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::post(async move |path, body| handler(path, body, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(NarMultipartsPartsPath { upload_id }): Path<
        NarMultipartsPartsPath<MultipartUploadId<Ctx>>,
    >,
    FromStrBody(body): FromStrBody<PartsRequestBody>,
    ctx: &mut Ctx,
) -> impl IntoResponse + use<Ctx> {
    // TODO: validate that the user has write permissions for that cache.

    let object_gateway = ctx.object_gateway();

    let storage_key = ObjectStorageKey::NarMultipart(&body.store_basename);

    let now = ctx.clock().unix_ts();

    let min_ttl = Duration::from_mins(10);

    let urls = body
        .part_numbers
        .into_iter()
        .map(|part_number| {
            object_gateway
                .multipart_upload_url(MultipartUploadUrlRequest {
                    key: storage_key,
                    part_number,
                    upload_id: &upload_id,
                    min_ttl,
                })
                .as_ref()
                .parse()
                .expect("valid URL")
        })
        .collect();

    (
        StatusCode::OK,
        PartsResponseBody { expiration_ts: now + min_ttl, urls }.to_string(),
    )
}
