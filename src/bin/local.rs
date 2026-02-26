//! TODO: docs.

use core::net::Ipv4Addr;

use cache::cdn::NoOpCdn;
use cache::context::ComposedContext;
use cache::object_gateway::IntoRouterObjectGateway;
use cache::object_storage::InMemoryObjectStorage;

#[tokio::main]
async fn main() {
    let object_gateway = IntoRouterObjectGateway::new(
        "/objects",
        InMemoryObjectStorage::default(),
    );

    let context = ComposedContext::new()
        .with_cdn(NoOpCdn::default())
        .with_object_gateway(object_gateway.clone());

    let router = cache::router(context).merge(object_gateway);

    let port = 3000;

    let tcp_listener =
        tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .await
            .expect("Failed to bind to socket");

    axum::serve(tcp_listener, router)
        .await
        .expect("Failed running local cache server");
}
