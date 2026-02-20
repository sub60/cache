use crate::cloudflare::CloudflareCdn;
use crate::context::Context;

/// The [`Context`] implementation for Cloudflare.
#[derive(Debug, Copy, Clone)]
pub struct CloudflareContext;

impl Context for CloudflareContext {
    type Cdn = CloudflareCdn;

    fn cdn(&self) -> &Self::Cdn {
        &CloudflareCdn
    }
}
