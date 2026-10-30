//! TODO: docs.

use core::{fmt, str};

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, MethodRouter};

use crate::context::{MultipartUploadId, ObjectStorageError};
use crate::from_str_body::FromStrBody;
use crate::nar_multiparts::CompleteRequestBody;
use crate::object_storage::ObjectStorageKey;
use crate::{CompleteMultipartUploadError, Context, ObjectStorage};

pub(crate) const ROUTE: &str = "/nar-multiparts/{upload_id}/complete";

#[derive(serde::Deserialize)]
struct NarMultipartsCompletePath<UploadId> {
    upload_id: UploadId,
}

#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
enum NarMultipartsCompleteError<ObjectStorageError> {
    #[display("couldn't abort redundant multipart upload: {_0}")]
    AbortMultipartUpload(ObjectStorageError),
    #[display("couldn't complete multipart upload: {_0}")]
    CompleteMultipartUpload(CompleteMultipartUploadError<ObjectStorageError>),
    #[display("multipart upload completion must include at least one part")]
    EmptyUpload,
    #[display("multipart upload has too many parts: {_0} (maximum {_1})")]
    TooManyParts(usize, usize),
}

pub(crate) fn router(mut ctx: impl Context) -> MethodRouter {
    routing::post(async move |path, body| handler(path, body, &mut ctx).await)
}

async fn handler<Ctx: Context>(
    Path(NarMultipartsCompletePath { upload_id }): Path<
        NarMultipartsCompletePath<MultipartUploadId<Ctx>>,
    >,
    FromStrBody(body): FromStrBody<CompleteRequestBody>,
    ctx: &mut Ctx,
) -> Result<StatusCode, NarMultipartsCompleteError<ObjectStorageError<Ctx>>> {
    // TODO: validate that the user has write permissions for that cache.

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
        .complete_multipart_upload(storage_key, upload_id.clone(), &body.etags)
        .await
        .map_err(NarMultipartsCompleteError::CompleteMultipartUpload)?;

    if did_insert {
        Ok(StatusCode::CREATED)
    } else {
        object_storage
            .abort_multipart_upload(storage_key, upload_id)
            .await
            .map_err(NarMultipartsCompleteError::AbortMultipartUpload)?;

        Ok(StatusCode::OK)
    }
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
            Self::AbortMultipartUpload(_)
            | Self::CompleteMultipartUpload(
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
