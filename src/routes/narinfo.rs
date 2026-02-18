//! TODO: docs.

use axum::extract::Path;
use axum::routing::{MethodRouter, get};

use crate::Context;
use crate::types::{CacheName, UserName};

pub(crate) const ROUTE: &str = "/{user}/{cache}/{hash}.narinfo";

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    get(async move |path| handler(path, &mut ctx).await)
}

#[derive(serde::Deserialize)]
#[expect(dead_code)]
struct NarInfoPath {
    user: UserName,
    cache: CacheName,
    hash: String,
}

async fn handler(Path(_path): Path<NarInfoPath>, _ctx: &mut impl Context) {}
