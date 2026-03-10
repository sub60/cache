use core::net::Ipv4Addr;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::http;
use types::NarFileName;

use crate::object_gateway::IntoRouterObjectGateway;
use crate::{ObjectGateway, ObjectStorage};

/// TODO: docs.
pub struct LoopbackObjectGateway<ObjStorage> {
    inner: IntoRouterObjectGateway<ObjStorage>,
    is_running: Arc<AtomicBool>,
    port: Option<u16>,
}

struct RunningGuard(Arc<AtomicBool>);

impl<ObjStorage> LoopbackObjectGateway<ObjStorage> {
    /// Creates a new [`LoopbackObjectGateway`] over the given object storage.
    pub fn new(storage: ObjStorage) -> Self {
        Self {
            inner: IntoRouterObjectGateway::new("/", storage),
            is_running: Arc::new(AtomicBool::new(false)),
            port: None,
        }
    }

    /// Returns a future that runs a local HTTP server on the given port,
    /// serving objects from the underlying object storage.
    pub fn run_server(
        &mut self,
        port: u16,
    ) -> impl Future<Output = ()> + Send + use<ObjStorage>
    where
        ObjStorage: Clone,
        IntoRouterObjectGateway<ObjStorage>: Into<axum::Router>,
    {
        if self.is_running.swap(true, Ordering::SeqCst) {
            panic!("LoopbackObjectGateway is already running");
        }

        self.port = Some(port);

        let is_running = Arc::clone(&self.is_running);

        let router = self.inner.clone().into();

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
{
    type Storage = ObjStorage;

    #[track_caller]
    fn nar_url(&self, key: &NarFileName) -> http::Uri {
        if !self.is_running.load(Ordering::SeqCst) {
            panic!(
                "LoopbackObjectGateway is not running. Please call \
                 `LoopbackObjectGateway::run()` and make sure the returned \
                 future is being polled"
            )
        }
        let port = self.port.expect("gateway is running, so port must be set");
        let relative_uri = self.inner.nar_url(key);
        format!("http://localhost:{port}/{relative_uri}")
            .parse()
            .expect("valid absolute URI")
    }

    fn storage(&self) -> &ObjStorage {
        self.inner.storage()
    }
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}
