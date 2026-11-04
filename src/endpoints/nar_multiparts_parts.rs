//! TODO: docs.

use core::time::Duration;

use axum::Extension;
use axum::http::StatusCode;
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
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let nar_storage = ctx.nar_storage();

    let now = ctx.clock().unix_ts();

    let min_ttl = Duration::from_mins(10);

    let urls = nar_storage
        .multipart_upload_part_urls(
            MultipartUpload {
                cache: &cache_infos,
                nar: &body.nar_file_name,
                id: &path.upload_id,
            },
            &body.part_numbers,
            min_ttl,
        )
        .await
        .map_err(|storage_err| {
            let message = storage_err.to_string();
            let status: StatusCode = storage_err.into();
            (status, message)
        })?;

    Ok((
        StatusCode::OK,
        nar_multiparts_parts::ResponseBody {
            expiration_ts: now + min_ttl,
            urls: urls
                .iter()
                .map(|url| url.as_ref().parse().expect("valid URL"))
                .collect(),
        }
        .to_string(),
    ))
}
