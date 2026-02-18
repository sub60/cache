//! TODO: docs.

use axum::extract::Path;
use axum::routing::{MethodRouter, get};

use crate::Context;

pub(crate) const ROUTE: &str = "/{user}/{cache}/{hash}.narinfo";

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    get(async move |path| handler(path, &mut ctx).await)
}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct NarInfoPath {
    user: String,
    cache: String,
    hash: String,
}

async fn handler(Path(_path): Path<NarInfoPath>, _ctx: &mut impl Context) {}
