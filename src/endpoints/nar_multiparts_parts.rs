//! TODO: docs.

use core::time::Duration;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_multiparts_parts;
use crate::context::MultipartUploadId;
use crate::from_str_body::FromStrBody;
use crate::object_gateway::MultipartUploadUrlRequest;
use crate::object_storage::ObjectStorageKey;
use crate::{Clock, Context, ObjectGateway, endpoints};

impl endpoints::Serve for nar_multiparts_parts::Endpoint {
    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(async move |path, body| {
            handler(path, body, &mut ctx).await
        })
    }
}

async fn handler<Ctx: Context>(
    path: nar_multiparts_parts::Path<MultipartUploadId<Ctx>>,
    FromStrBody(body): FromStrBody<nar_multiparts_parts::RequestBody>,
    ctx: &mut Ctx,
) -> impl IntoResponse + use<Ctx> {
    // TODO: validate that the user has write permissions for that cache.

    let object_gateway = ctx.object_gateway();

    let storage_key = ObjectStorageKey::Nar(&body.nar_file_name);

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
                    upload_id: &path.upload_id,
                    min_ttl,
                })
                .as_ref()
                .parse()
                .expect("valid URL")
        })
        .collect();

    (
        StatusCode::OK,
        nar_multiparts_parts::ResponseBody {
            expiration_ts: now + min_ttl,
            urls,
        }
        .to_string(),
    )
}
