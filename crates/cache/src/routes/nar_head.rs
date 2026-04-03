//! TODO: docs.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::routes::nar_put::NarPutPath;
use crate::{Context, ObjectStorage};

pub(crate) const ROUTE: &str = crate::routes::nar_put::ROUTE;

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::head(async move |path| handler(path, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(NarPutPath { nar_file_name }): Path<NarPutPath>,
    ctx: &mut Ctx,
) -> Result<StatusCode, impl IntoResponse + use<Ctx>> {
    ctx.object_storage()
        .contains_key(&nar_file_name)
        .await
        .map(
            |exists| {
                if exists { StatusCode::OK } else { StatusCode::NOT_FOUND }
            },
        )
        .map_err(|storage_err| {
            (StatusCode::INTERNAL_SERVER_ERROR, storage_err.to_string())
        })
}
