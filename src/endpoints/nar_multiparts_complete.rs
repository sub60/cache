//! TODO: docs.

use core::fmt;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;

use crate::api::endpoints::nar_multiparts_complete;
use crate::context::{MultipartUploadId, ObjectStorageError};
use crate::from_str_body::FromStrBody;
use crate::object_storage::ObjectStorageKey;
use crate::{
    CompleteMultipartUploadError,
    Context,
    ObjectStorage,
    Permission,
    endpoints,
};

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarMultipartsCompleteError<ObjectStorageError> {
    #[display("couldn't complete multipart upload: {_0}")]
    CompleteMultipartUpload(CompleteMultipartUploadError<ObjectStorageError>),
    #[display("multipart upload completion must include at least one part")]
    EmptyUpload,
    #[display("multipart upload has too many parts: {_0} (maximum {_1})")]
    TooManyParts(usize, usize),
}

impl endpoints::CacheEndpoint for nar_multiparts_complete::Endpoint {
    const PERMISSION: Permission = Permission::Write;

    fn method_router(mut ctx: impl Context) -> MethodRouter {
        endpoints::method_router::<Self, _, _>(async move |path, body| {
            handler(path, body, &mut ctx)
                .await
                .map(|response| response.status_code())
        })
    }
}

async fn handler<Ctx: Context>(
    path: nar_multiparts_complete::Path<MultipartUploadId<Ctx>>,
    FromStrBody(body): FromStrBody<nar_multiparts_complete::RequestBody>,
    ctx: &mut Ctx,
) -> Result<
    nar_multiparts_complete::Response,
    NarMultipartsCompleteError<ObjectStorageError<Ctx>>,
> {
    if body.etags.is_empty() {
        return Err(NarMultipartsCompleteError::EmptyUpload);
    }

    let object_storage = ctx.object_storage();

    if let Some(max_multipart_parts) = object_storage.max_multipart_parts()
        && body.etags.len() > max_multipart_parts
    {
        return Err(NarMultipartsCompleteError::TooManyParts(
            body.etags.len(),
            max_multipart_parts,
        ));
    }

    let storage_key = ObjectStorageKey::Nar(&body.nar_file_name);

    let did_insert = object_storage
        .complete_multipart_upload(
            storage_key,
            path.upload_id.clone(),
            &body.etags,
        )
        .await
        .map_err(NarMultipartsCompleteError::CompleteMultipartUpload)?;

    if did_insert {
        return Ok(nar_multiparts_complete::Response::Created);
    }

    // The NAR is already stored, so aborting this upload is just cleanup, and
    // failing to do so shouldn't fail the request.
    if let Err(err) = object_storage
        .abort_multipart_upload(storage_key, path.upload_id.clone())
        .await
    {
        tracing::warn!(
            upload_id = %path.upload_id,
            nar_file_name = %body.nar_file_name,
            "couldn't abort redundant multipart upload: {err}",
        );
    }

    Ok(nar_multiparts_complete::Response::AlreadyExists)
}

impl<ObjectStorageError> NarMultipartsCompleteError<ObjectStorageError> {
    fn status_code(&self) -> StatusCode {
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
                CompleteMultipartUploadError::Storage(_),
            ) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<ObjectStorageError: fmt::Display> IntoResponse
    for NarMultipartsCompleteError<ObjectStorageError>
{
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
