use core::fmt;
use std::borrow::Cow;

use axum::Router;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use nix_types::{NarFileName, NarInfoFileName};
use sub60_cache::{
    MultipartUploadUrlRequest,
    ObjectGateway,
    ObjectStorage,
    ObjectStorageKey,
    ObjectUrlRequest,
};

/// TODO: docs.
#[derive(Clone)]
pub struct IntoRouterObjectGateway<ObjStorage> {
    base_route: Cow<'static, str>,
    storage: ObjStorage,
}

impl<ObjStorage> IntoRouterObjectGateway<ObjStorage> {
    /// Creates a new [`IntoRouterObjectGateway`] over the given object storage
    /// handling requests at the given route path.
    ///
    /// # Panics
    ///
    /// Panics if the base route path does not start with a slash or if it
    /// contains path captures (e.g., `/{foo}`).
    pub fn new(base_route: &str, storage: ObjStorage) -> Self {
        assert!(
            base_route.starts_with('/'),
            "route path must start with a slash"
        );
        assert!(
            !base_route.contains('{') && !base_route.contains('}'),
            "base route must not contain path captures (`{{...}}`)"
        );
        let base_route = if base_route == "/" {
            Cow::Borrowed("")
        } else {
            Cow::Owned(base_route.trim_end_matches('/').to_owned())
        };
        Self { base_route, storage }
    }
}

impl<ObjStorage: ObjectStorage> From<IntoRouterObjectGateway<ObjStorage>>
    for Router
where
    ObjStorage: ObjectStorage + Clone + Send + 'static,
    ObjStorage::Error: fmt::Display,
{
    fn from(gateway: IntoRouterObjectGateway<ObjStorage>) -> Self {
        let storage = gateway.storage.clone();

        let get_handler = async move |Path::<String>(key)| {
            let object_key = match object_key_from_str(&key) {
                Ok(key) => key,
                Err(err) => {
                    let msg = err.to_string();
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            match storage.get(&object_key).await {
                Ok(Some(object)) => {
                    Response::new(object.as_ref().to_owned().into())
                },
                Ok(None) => StatusCode::NOT_FOUND.into_response(),
                Err(err) => {
                    let msg = err.to_string();
                    (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
                },
            }
        };

        let put_handler = async move |Path::<String>(key), object: Bytes| {
            let object_key = match object_key_from_str(&key) {
                Ok(key) => key,
                Err(err) => {
                    let msg = err.to_string();
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            match gateway.storage.insert(&object_key, &object).await {
                Ok(true) => StatusCode::CREATED.into_response(),
                Ok(false) => StatusCode::NO_CONTENT.into_response(),
                Err(err) => {
                    let msg = err.to_string();
                    (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
                },
            }
        };

        let multipart_put_handler = async || StatusCode::NOT_IMPLEMENTED;

        let route_path = if gateway.base_route.is_empty() {
            Cow::Borrowed("/{*key}")
        } else {
            Cow::Owned(format!("{}/{{*key}}", gateway.base_route))
        };

        let multipart_route_path = if gateway.base_route.is_empty() {
            Cow::Borrowed("/multipart-upload/{*key}")
        } else {
            Cow::Owned(format!(
                "{}/multipart-upload/{{*key}}",
                gateway.base_route,
            ))
        };

        Self::new()
            .route(&multipart_route_path, put(multipart_put_handler))
            .route(&route_path, get(get_handler))
            .route(&route_path, put(put_handler))
    }
}

impl<ObjStorage> ObjectGateway for IntoRouterObjectGateway<ObjStorage>
where
    ObjStorage: ObjectStorage,
    ObjStorage::MultipartUploadId: fmt::Display,
{
    type Storage = ObjStorage;

    fn object_url(&self, access_request: ObjectUrlRequest) -> impl AsRef<str> {
        let key = object_key_display(access_request.key);

        if self.base_route.is_empty() {
            format!("/{key}")
        } else {
            format!("{}/{key}", self.base_route)
        }
    }

    fn multipart_upload_url(
        &self,
        request: MultipartUploadUrlRequest<
            '_,
            <Self::Storage as ObjectStorage>::MultipartUploadId,
        >,
    ) -> impl AsRef<str> {
        let key = object_key_display(request.key);

        let base_url = if self.base_route.is_empty() {
            format!("/multipart-upload/{key}")
        } else {
            format!("{}/multipart-upload/{key}", self.base_route)
        };

        format!(
            "http://cache.test{base_url}?partNumber={}&uploadId={}",
            request.part_number, request.upload_id,
        )
    }

    fn storage(&self) -> &ObjStorage {
        &self.storage
    }
}

fn object_key_display(key: ObjectStorageKey) -> impl fmt::Display {
    match key {
        ObjectStorageKey::Nar(nar_file_name) => {
            format!("nar/{nar_file_name}")
        },
        ObjectStorageKey::NarInfo(narinfo_file_name) => {
            format!("narinfo/{narinfo_file_name}")
        },
    }
}

fn object_key_from_str(s: &str) -> Result<OwnedObjectStorageKey, String> {
    let (key_kind, key) =
        s.split_once('/').ok_or_else(|| format!("invalid key format: {s}"))?;

    match key_kind {
        "nar" => key
            .parse()
            .map(OwnedObjectStorageKey::Nar)
            .map_err(|err| format!("invalid NarFileName in key: {err}")),
        "narinfo" => key
            .parse()
            .map(OwnedObjectStorageKey::NarInfo)
            .map_err(|err| format!("invalid NarInfoFileName in key: {err}")),
        _ => Err(format!("invalid key kind: {key_kind}")),
    }
}

enum OwnedObjectStorageKey {
    Nar(NarFileName),
    NarInfo(NarInfoFileName),
}

impl<'a> From<&'a OwnedObjectStorageKey> for ObjectStorageKey<'a> {
    fn from(object_key: &'a OwnedObjectStorageKey) -> Self {
        match object_key {
            OwnedObjectStorageKey::Nar(nar_file_name) => {
                Self::Nar(nar_file_name)
            },
            OwnedObjectStorageKey::NarInfo(narinfo_file_name) => {
                Self::NarInfo(narinfo_file_name)
            },
        }
    }
}
