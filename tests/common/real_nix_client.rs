use core::convert::Infallible;
use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use core::str::FromStr;

use crate::common::{CopyOptions, NixClient};

/// TODO: docs.
pub struct RealNixClient;

/// TODO: docs.
#[derive(Debug)]
#[expect(unused)]
pub enum CacheHost {
    /// TODO: docs.
    LocalCache(SocketAddr),
    /// TODO: docs.
    RemoteCache(url::Url),
}

impl NixClient for RealNixClient {
    type CacheHost = CacheHost;
    type Installable = String;
    type CopyError = Infallible;

    async fn copy_impl(
        &self,
        _installables: Vec<Self::Installable>,
        options: CopyOptions<true, Self>,
    ) -> Result<(), Self::CopyError> {
        let (_app, _user, _cache) = options.cache_uri();
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

impl TryFrom<&str> for CacheHost {
    type Error = <url::Url as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse::<url::Url>().map(Self::RemoteCache)
    }
}
