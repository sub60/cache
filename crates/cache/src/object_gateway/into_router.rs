use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

use axum::Router;
use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{self, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use types::{NarFileName, NarInfoFileName};

use crate::object_storage::ObjectStorageKey;
use crate::{ObjectGateway, ObjectStorage};

/// TODO: docs.
#[derive(Clone)]
pub struct IntoRouterObjectGateway<ObjStorage> {
    base_route: Cow<'static, str>,
    storage: ObjStorage,
}

enum ObjectKey {
    Nar(NarFileName),
    NarInfo(NarInfoFileName),
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
            let object_key = match ObjectKey::from_str(&key) {
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
            let object_key = match ObjectKey::from_str(&key) {
                Ok(key) => key,
                Err(err) => {
                    let msg = err.to_string();
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            match gateway.storage.set(&object_key, &object).await {
                Ok(()) => StatusCode::CREATED.into_response(),
                Err(err) => {
                    let msg = err.to_string();
                    (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
                },
            }
        };

        let route_path = if gateway.base_route.is_empty() {
            Cow::Borrowed("/{key}")
        } else {
            Cow::Owned(format!("{}/{{key}}", gateway.base_route))
        };

        Self::new()
            .route(&route_path, get(get_handler))
            .route(&route_path, put(put_handler))
    }
}

impl<ObjStorage: ObjectStorage> ObjectGateway
    for IntoRouterObjectGateway<ObjStorage>
{
    type Storage = ObjStorage;

    fn nar_url<const IS_FOR_PUT: bool>(
        &self,
        filename: &NarFileName,
    ) -> http::Uri {
        let key = ObjectKey::Nar(filename.clone());
        if self.base_route.is_empty() {
            format!("/{key}")
        } else {
            format!("{}/{key}", self.base_route)
        }
        .parse()
        .expect("valid relative URI")
    }

    fn storage(&self) -> &ObjStorage {
        &self.storage
    }
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nar(nar_file_name) => write!(f, "nar-{nar_file_name}"),
            Self::NarInfo(narinfo_file_name) => {
                write!(f, "narinfo-{narinfo_file_name}")
            },
        }
    }
}

impl FromStr for ObjectKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key_kind, key) = s
            .split_once('-')
            .ok_or_else(|| format!("invalid key format: {s}"))?;

        match key_kind {
            "nar" => key
                .parse()
                .map(Self::Nar)
                .map_err(|err| format!("invalid NarFileName in key: {err}")),
            "narinfo" => key.parse().map(Self::NarInfo).map_err(|err| {
                format!("invalid NarInfoFileName in key: {err}")
            }),
            _ => Err(format!("invalid key kind: {key_kind}")),
        }
    }
}

impl<'a> From<&'a ObjectKey> for ObjectStorageKey<'a> {
    fn from(object_key: &'a ObjectKey) -> Self {
        match object_key {
            ObjectKey::Nar(nar_file_name) => Self::Nar(nar_file_name),
            ObjectKey::NarInfo(narinfo_file_name) => {
                Self::NarInfo(narinfo_file_name)
            },
        }
    }
}
