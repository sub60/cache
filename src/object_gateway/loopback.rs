use core::fmt;
use core::net::Ipv4Addr;
use core::str::FromStr;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::Path;
use axum::http::{self, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;

use crate::{ObjectGateway, ObjectStorage};

/// TODO: docs.
pub struct LoopbackObjectGateway<ObjStorage> {
    is_running: Arc<AtomicBool>,
    port: Option<u16>,
    storage: ObjStorage,
}

struct RunningGuard(Arc<AtomicBool>);

impl<ObjStorage> LoopbackObjectGateway<ObjStorage> {
    /// Creates a new [`LoopbackObjectGateway`] over the given object storage.
    pub fn new(storage: ObjStorage) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            port: None,
            storage,
        }
    }

    /// Returns a future that runs a local HTTP server on the given port,
    /// serving objects from the underlying object storage.
    pub fn run_server(
        &mut self,
        port: u16,
    ) -> impl Future<Output = ()> + Send + use<ObjStorage>
    where
        ObjStorage: ObjectStorage + Clone + Send + 'static,
        ObjStorage::Error: fmt::Display,
        for<'a> ObjStorage::Key<'a>: FromStr<Err: fmt::Display>,
    {
        if self.is_running.swap(true, Ordering::SeqCst) {
            panic!("LoopbackObjectGateway is already running");
        }

        self.port = Some(port);

        let object_storage = self.storage.clone();

        let handler = async move |Path::<String>(key)| {
            let object_key = match ObjStorage::Key::from_str(&key) {
                Ok(key) => key,
                Err(err) => {
                    let msg = err.to_string();
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                },
            };
            match object_storage.get(&object_key).await {
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

        let router = axum::Router::new().route("/{key}", get(handler));

        let is_running = Arc::clone(&self.is_running);

        async move {
            let _running_guard = RunningGuard(is_running);

            let socket_addr = (Ipv4Addr::LOCALHOST, port);

            let listener = tokio::net::TcpListener::bind(socket_addr)
                .await
                .expect("Failed to bind to socket");

            axum::serve(listener, router)
                .await
                .expect("Failed to start LoopbackObjectGateway");
        }
    }
}

impl<ObjStorage: ObjectStorage> ObjectGateway
    for LoopbackObjectGateway<ObjStorage>
where
    for<'a> ObjStorage::Key<'a>: fmt::Display,
{
    type Storage = ObjStorage;

    #[track_caller]
    fn object_url(&self, key: ObjStorage::Key<'_>) -> http::Uri {
        if !self.is_running.load(Ordering::SeqCst) {
            panic!(
                "LoopbackObjectGateway is not running. Please call \
                 `LoopbackObjectGateway::run()` and make sure the returned \
                 future is being polled"
            )
        }
        let port = self.port.expect("gateway is running, so port must be set");
        format!("http://localhost:{port}/{}", key).parse().expect("valid URI")
    }

    fn storage(&self) -> &ObjStorage {
        &self.storage
    }
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
