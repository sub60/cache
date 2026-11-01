//! TODO: docs.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_head;
use crate::{Context, ObjectStorage, ObjectStorageKey, endpoints};

impl endpoints::Serve for nar_head::Endpoint {
    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(async move |path| {
            handler(path, &mut ctx).await
        })
    }
}

async fn handler<Ctx: Context>(
    path: nar_head::Path,
    ctx: &mut Ctx,
) -> Result<StatusCode, impl IntoResponse + use<Ctx>> {
    ctx.object_storage()
        .contains_key(ObjectStorageKey::Nar(&path.nar_file_name))
        .await
        .map(|exists| {
            let response = if exists {
                nar_head::Response::Exists
            } else {
                nar_head::Response::NotFound
            };
            response.status_code()
        })
        .map_err(|storage_err| {
            (StatusCode::INTERNAL_SERVER_ERROR, storage_err.to_string())
        })
}
