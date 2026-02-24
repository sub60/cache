use crate::cloudflare::{CloudflareCdn, CloudflareObjectStorage};
use crate::context::Context;

/// The [`Context`] implementation for Cloudflare.
#[derive(Debug, Copy, Clone)]
pub struct CloudflareContext;

impl Context for CloudflareContext {
    type Cdn = CloudflareCdn;
    type ObjectStorage = CloudflareObjectStorage;

    fn cdn(&self) -> &Self::Cdn {
        &CloudflareCdn
    }

    fn object_storage(&self) -> &Self::ObjectStorage {
        &CloudflareObjectStorage
    }
}
