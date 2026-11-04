//! TODO: docs.

use core::fmt;

use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_multiparts_complete;
use crate::context::{MultipartUploadId, NarStorageError};
use crate::from_str_body::FromStrBody;
use crate::{
    CacheInfos,
    CompleteMultipartUploadError,
    Context,
    MultipartUpload,
    NarStorage,
    Permission,
    Permissions,
    endpoints,
};

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarMultipartsCompleteError<NarStorageError> {
    #[display("couldn't complete multipart upload: {_0}")]
    CompleteMultipartUpload(CompleteMultipartUploadError<NarStorageError>),
    #[display("multipart upload completion must include at least one part")]
    EmptyUpload,
    #[display("multipart upload has too many parts: {_0} (maximum {_1})")]
    TooManyParts(usize, usize),
}

impl endpoints::CacheEndpoint for nar_multiparts_complete::Endpoint {
    const PERMISSIONS: Permissions =
        Permissions::new([Permission::PushToCache]);

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(
            async move |path, Extension(cache_infos), body| {
                handler(path, cache_infos, body, &mut ctx)
                    .await
                    .map(|response| response.status_code())
            },
        )
    }
}

async fn handler<Ctx: Context>(
    path: nar_multiparts_complete::Path<MultipartUploadId<Ctx>>,
    cache_infos: CacheInfos,
    FromStrBody(body): FromStrBody<nar_multiparts_complete::RequestBody>,
    ctx: &mut Ctx,
) -> Result<
    nar_multiparts_complete::Response,
    NarMultipartsCompleteError<NarStorageError<Ctx>>,
> {
    if body.etags.is_empty() {
        return Err(NarMultipartsCompleteError::EmptyUpload);
    }

    let nar_storage = ctx.nar_storage();

    if let Some(max_multipart_parts) = nar_storage.max_multipart_parts()
        && body.etags.len() > max_multipart_parts
    {
        return Err(NarMultipartsCompleteError::TooManyParts(
            body.etags.len(),
            max_multipart_parts,
        ));
    }

    let upload = MultipartUpload {
        cache: &cache_infos,
        nar: &body.nar_file_name,
        id: &path.upload_id,
    };

    let did_insert = nar_storage
        .complete_multipart_upload(upload, &body.etags)
        .await
        .map_err(NarMultipartsCompleteError::CompleteMultipartUpload)?;

    if did_insert {
        return Ok(nar_multiparts_complete::Response::Created);
    }

    // The NAR is already stored, so aborting this upload is just cleanup, and
    // failing to do so shouldn't fail the request.
    if let Err(err) = nar_storage.abort_multipart_upload(upload).await {
        tracing::warn!(
            upload_id = %path.upload_id,
            nar_file_name = %body.nar_file_name,
            "couldn't abort redundant multipart upload: {err}",
        );
    }

    Ok(nar_multiparts_complete::Response::AlreadyExists)
}

impl<NarStorageError: Into<StatusCode>>
    NarMultipartsCompleteError<NarStorageError>
{
    fn into_status_code(self) -> StatusCode {
        match self {
            Self::CompleteMultipartUpload(
                CompleteMultipartUploadError::NoSuchUpload,
            ) => StatusCode::NOT_FOUND,
            Self::CompleteMultipartUpload(
                CompleteMultipartUploadError::InvalidParts(_),
            )
            | Self::EmptyUpload
            | Self::TooManyParts(..) => StatusCode::BAD_REQUEST,
            Self::CompleteMultipartUpload(
                CompleteMultipartUploadError::Storage(err),
            ) => err.into(),
        }
    }
}

impl<NarStorageError> IntoResponse
    for NarMultipartsCompleteError<NarStorageError>
where
    NarStorageError: fmt::Display + Into<StatusCode>,
{
    fn into_response(self) -> axum::response::Response {
        let message = self.to_string();
        (self.into_status_code(), message).into_response()
    }
}
