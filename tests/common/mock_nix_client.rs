use core::convert::Infallible;

use crate::common::{CopyOptions, NixClient};

/// TODO: docs.
pub struct MockNixClient;

/// TODO: docs.
#[expect(unused)]
pub struct NarBytes(Vec<u8>);

impl NixClient for MockNixClient {
    type CacheHost = axum::Router;
    type Installable = NarBytes;
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

impl<T: AsRef<[u8]>> From<T> for NarBytes {
    fn from(bytes: T) -> Self {
        Self(bytes.as_ref().to_owned())
    }
}
