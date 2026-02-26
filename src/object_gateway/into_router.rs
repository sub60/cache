use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

use axum::Router;
use axum::extract::Path;
use axum::http::{self, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;

use crate::{ObjectGateway, ObjectStorage};

/// TODO: docs.
#[derive(Clone)]
pub struct IntoRouterObjectGateway<ObjStorage> {
    route_path: Cow<'static, str>,
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
        let route_path = if base_route == "/" {
            Cow::Borrowed("/{key}")
        } else {
            Cow::Owned(format!("{}/{{key}}", base_route.trim_end_matches('/')))
        };
        Self { route_path, storage }
    }
}

impl<ObjStorage: ObjectStorage> From<IntoRouterObjectGateway<ObjStorage>>
    for Router
where
    ObjStorage: ObjectStorage + Clone + Send + 'static,
    ObjStorage::Error: fmt::Display,
    for<'a> ObjStorage::Key<'a>: FromStr<Err: fmt::Display>,
{
    fn from(gateway: IntoRouterObjectGateway<ObjStorage>) -> Self {
        let handler = async move |Path::<String>(key)| {
            let object_key = match ObjStorage::Key::from_str(&key) {
                Ok(key) => key,
                Err(err) => {
                    let msg = err.to_string();
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            match gateway.storage.get(&object_key).await {
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

        Self::new().route(&gateway.route_path, get(handler))
    }
}

impl<ObjStorage: ObjectStorage> ObjectGateway
    for IntoRouterObjectGateway<ObjStorage>
where
    for<'a> ObjStorage::Key<'a>: fmt::Display,
{
    type Storage = ObjStorage;

    fn object_url(&self, key: ObjStorage::Key<'_>) -> http::Uri {
        format!("{}/{}", self.route_path, key)
            .parse()
            .expect("valid relative URI")
    }

    fn storage(&self) -> &ObjStorage {
        &self.storage
    }
}
