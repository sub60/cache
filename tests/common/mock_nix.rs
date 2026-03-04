use core::convert::Infallible;

use cache::types::NixStorePath;

use crate::common::{CopyOptions, NixClient, NixStore};

/// TODO: docs.
#[derive(Default)]
pub struct MockNixClient {
    #[expect(unused)]
    store: MockNixStore,
}

/// TODO: docs.
#[derive(Default)]
pub struct MockNixStore;

/// TODO: docs.
#[expect(unused)]
pub struct NarBytes(Vec<u8>);

impl NixClient for MockNixClient {
    type Store = MockNixStore;
    type CacheHost = axum::Router;
    type CopyError = Infallible;

    fn store(&self) -> &Self::Store {
        &MockNixStore
    }

    async fn copy_impl(
        &self,
        _store_paths: Vec<NixStorePath>,
        options: CopyOptions<true, Self>,
    ) -> Result<(), Self::CopyError> {
        let (_app, _user, _cache) = options.cache_uri();
        todo!()
    }
}

impl NixStore for MockNixStore {
    async fn add_file(
        &self,
        _file_name: &str,
        _file_contents: &[u8],
    ) -> NixStorePath {
        todo!()
    }
}

impl<T: AsRef<[u8]>> From<T> for NarBytes {
    fn from(bytes: T) -> Self {
        Self(bytes.as_ref().to_owned())
    }
}
