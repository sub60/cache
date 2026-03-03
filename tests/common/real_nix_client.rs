use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use core::str::FromStr;

use axum::Error;

use crate::common::{CopyOptions, NixClient};

/// TODO: docs.
pub struct RealNixClient;

/// TODO: docs.
pub enum CacheHost {
    /// TODO: docs.
    LocalCache(SocketAddr),
    /// TODO: docs.
    RemoteCache(url::Url),
}

impl NixClient for RealNixClient {
    type CacheHost = axum::Router;
    type Installable = NarBytes;
    type CopyError;

    async fn copy_impl(
        &self,
        installables: Vec<Self::Installable>,
        options: CopyOptions<true, Self>,
    ) -> Result<(), Self::CopyError> {
        let (_app, user, cache) = options
            .from
            .ok_or(options.to)
            .expect("exactly one of `from` and `to` must be set");
        todo!()
    }
}

impl From<SocketAddrV4> for CacheHost {
    fn from(addr: SocketAddrV4) -> Self {
        SocketAddr::V4(addr).into()
    }
}

impl From<SocketAddrV6> for CacheHost {
    fn from(addr: SocketAddrV6) -> Self {
        SocketAddr::V6(addr).into()
    }
}

impl From<SocketAddr> for CacheHost {
    fn from(addr: SocketAddr) -> Self {
        Self::LocalCache(addr)
    }
}

impl From<url::Url> for CacheHost {
    fn from(url: url::Url) -> Self {
        Self::RemoteCache(url)
    }
}

impl<T: AsRef<str>> TryFrom<T> for CacheHost {
    type Error = <url::Url as FromStr>::Error;

    fn try_from(value: T) -> Result<Self, Self::Error> {
        value.as_ref().parse::<url::Url>().map(Self::RemoteCache)
    }
}
