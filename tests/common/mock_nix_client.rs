use crate::common::{CopyOptions, NixClient};

/// TODO: docs.
pub struct MockNixClient;

impl NixClient for MockNixClient {
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
