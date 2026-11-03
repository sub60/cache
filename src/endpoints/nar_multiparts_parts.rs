//! TODO: docs.

use core::time::Duration;

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_multiparts_parts;
use crate::context::MultipartUploadId;
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

impl endpoints::CacheEndpoint for nar_multiparts_parts::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PushToCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |path, Extension(cache_infos), body| {
                handler(path, cache_infos, body, &mut ctx).await
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: nar_multiparts_parts::Path<MultipartUploadId<Ctx>>,
    cache_infos: CacheInfos,
    FromStrBody(body): FromStrBody<nar_multiparts_parts::RequestBody>,
    ctx: &mut Ctx,
) -> impl IntoResponse + use<Ctx> {
    let nar_storage = ctx.nar_storage();

    let now = ctx.clock().unix_ts();

    let min_ttl = Duration::from_mins(10);

    let urls = body
        .part_numbers
        .into_iter()
        .map(|part_number| {
            nar_storage
                .multipart_upload_part_url(
                    MultipartUpload {
                        cache_id: &cache_infos.id,
                        nar: &body.nar_file_name,
                        id: &path.upload_id,
                    },
                    part_number,
                    min_ttl,
                )
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
