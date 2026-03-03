use core::error::Error;
use core::fmt;

use cache::types::{CacheName, UserName};

/// TODO: docs.
pub trait NixClient {
    /// TODO: docs.
    type CacheHost;

    /// TODO: docs.
    type Installable: Send;

    /// TODO: docs.
    type CopyError: Error + Send;

    /// TODO: docs.
    fn copy_impl(
        &self,
        installables: Vec<Self::Installable>,
        options: CopyOptions<true, Self>,
    ) -> impl Future<Output = Result<(), Self::CopyError>>;

    /// TODO: docs.
    fn copy(
        &self,
        installables: impl IntoIterator<Item = impl Into<Self::Installable>>,
    ) -> CopyFutureBuilder<'_, false, Self> {
        CopyFutureBuilder {
            client: self,
            installables: installables.into_iter().map(Into::into).collect(),
            opts: CopyOptions { from: None, to: None },
        }
    }
}

/// TODO: docs.
pub struct CopyOptions<const TO_XOR_FROM: bool, Client: NixClient> {
    /// TODO: docs.
    pub from: Option<(Client::CacheHost, UserName, CacheName)>,
    /// TODO: docs.
    pub to: Option<(Client::CacheHost, UserName, CacheName)>,
}

/// TODO: docs.
pub struct CopyFutureBuilder<
    'client,
    const TO_XOR_FROM: bool,
    Client: NixClient,
> {
    client: &'client Client,
    installables: Vec<Client::Installable>,
    opts: CopyOptions<TO_XOR_FROM, Client>,
}

impl<'client, Client: NixClient> CopyFutureBuilder<'client, false, Client> {
    /// TODO: docs.
    #[track_caller]
    pub fn from(
        mut self,
        cache_host: impl Into<Client::CacheHost>,
        user_name: impl TryInto<UserName, Error: fmt::Debug>,
        cache_name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CopyFutureBuilder<'client, true, Client> {
        let user_name = user_name.try_into().expect("invalid user name");
        let cache_name = cache_name.try_into().expect("invalid cache name");
        let opts = CopyOptions {
            from: Some((cache_host.into(), user_name, cache_name)),
            to: None,
        };
        CopyFutureBuilder {
            client: self.client,
            installables: self.installables,
            opts,
        }
    }

    /// TODO: docs.
    #[track_caller]
    pub fn to(
        mut self,
        cache_host: impl Into<Client::CacheHost>,
        user_name: impl TryInto<UserName, Error: fmt::Debug>,
        cache_name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CopyFutureBuilder<'client, true, Client> {
        let user_name = user_name.try_into().expect("invalid user name");
        let cache_name = cache_name.try_into().expect("invalid cache name");
        let opts = CopyOptions {
            from: None,
            to: Some((cache_host.into(), user_name, cache_name)),
        };
        CopyFutureBuilder {
            client: self.client,
            installables: self.installables,
            opts,
        }
    }
}

impl<'client, Client: NixClient> IntoFuture
    for CopyFutureBuilder<'client, true, Client>
{
    type Output = Result<(), Client::CopyError>;
    type IntoFuture = Box<dyn Future<Output = Self::Output> + Send + 'client>;

    fn into_future(self) -> Self::IntoFuture {
        Box::new(self.client.copy_impl(self.installables, self.opts))
    }
}
