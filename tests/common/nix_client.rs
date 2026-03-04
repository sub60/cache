use core::error::Error;
use core::fmt;
use core::pin::Pin;

use cache::types::{CacheName, NixStorePath, UserName};

use crate::common::NixStore;

/// TODO: docs.
pub trait NixClient: Sized {
    /// TODO: docs.
    type Store: NixStore;

    /// TODO: docs.
    type CacheHost;

    /// TODO: docs.
    type CopyError: Error + Send;

    /// TODO: docs.
    fn copy_impl(
        &self,
        store_paths: Vec<NixStorePath>,
        options: CopyOptions<true, Self>,
    ) -> impl Future<Output = Result<(), Self::CopyError>>;

    fn store(&self) -> &Self::Store;

    /// TODO: docs.
    fn copy(
        &self,
        store_paths: impl IntoIterator<Item = impl Into<NixStorePath>>,
    ) -> CopyFutureBuilder<'_, false, Self> {
        CopyFutureBuilder {
            client: self,
            store_paths: store_paths.into_iter().map(Into::into).collect(),
            opts: CopyOptions {
                cache_uri: None,
                direction: CopyDirection::From,
            },
        }
    }
}

/// TODO: docs.
pub struct CopyOptions<const HAS_CACHE_URI: bool, Client: NixClient> {
    pub cache_uri:
        Option<(<Client as NixClient>::CacheHost, UserName, CacheName)>,
    pub direction: CopyDirection,
}

#[derive(Debug, Copy, Clone)]
pub enum CopyDirection {
    From,
    To,
}

/// TODO: docs.
pub struct CopyFutureBuilder<
    'client,
    const HAS_CACHE_URI: bool,
    Client: NixClient,
> {
    client: &'client Client,
    store_paths: Vec<NixStorePath>,
    opts: CopyOptions<HAS_CACHE_URI, Client>,
}

impl<'client, Client: NixClient> CopyFutureBuilder<'client, false, Client> {
    /// TODO: docs.
    #[track_caller]
    #[expect(dead_code)]
    pub fn from(
        mut self,
        cache_host: impl Into<Client::CacheHost>,
        user_name: impl TryInto<UserName, Error: fmt::Debug>,
        cache_name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CopyFutureBuilder<'client, true, Client> {
        self.opts.direction = CopyDirection::From;
        CopyFutureBuilder {
            client: self.client,
            store_paths: self.store_paths,
            opts: self.opts.with_cache_uri(cache_host, user_name, cache_name),
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
        self.opts.direction = CopyDirection::To;
        CopyFutureBuilder {
            client: self.client,
            store_paths: self.store_paths,
            opts: self.opts.with_cache_uri(cache_host, user_name, cache_name),
        }
    }
}

impl<Client: NixClient> CopyOptions<false, Client> {
    fn with_cache_uri(
        self,
        cache_host: impl Into<Client::CacheHost>,
        user_name: impl TryInto<UserName, Error: fmt::Debug>,
        cache_name: impl TryInto<CacheName, Error: fmt::Debug>,
    ) -> CopyOptions<true, Client> {
        let user_name = user_name.try_into().expect("invalid user name");
        let cache_name = cache_name.try_into().expect("invalid cache name");
        CopyOptions {
            cache_uri: Some((cache_host.into(), user_name, cache_name)),
            direction: CopyDirection::From,
        }
    }
}

impl<Client: NixClient> CopyOptions<true, Client> {
    pub fn cache_uri(
        self,
    ) -> (<Client as NixClient>::CacheHost, UserName, CacheName) {
        self.cache_uri.expect("cache URI is set")
    }

    pub fn into_cli_args(self) -> impl Iterator<Item = impl AsRef<str>>
    where
        Client::CacheHost: Into<url::Url>,
    {
        let direction = self.direction;

        let (host, user, cache) = self.cache_uri();

        let mut cache_url = host.into();

        {
            let mut segments = cache_url
                .path_segments_mut()
                .expect("remote cache URL cannot be a base");
            segments.push(&user);
            segments.push(&cache);
        }

        vec![direction.as_cli_arg().to_owned(), cache_url.to_string()]
            .into_iter()
    }
}

impl CopyDirection {
    fn as_cli_arg(self) -> &'static str {
        match self {
            Self::From => "--from",
            Self::To => "--to",
        }
    }
}

impl<'client, Client: NixClient> IntoFuture
    for CopyFutureBuilder<'client, true, Client>
{
    type Output = Result<(), Client::CopyError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'client>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.client.copy_impl(self.store_paths, self.opts))
    }
}
