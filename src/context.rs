//! TODO: docs.

use rand_core::CryptoRng;

use crate::{
    CacheDb,
    Cdn,
    Clock,
    ObjectGateway,
    ObjectStorage,
    OidcVerifier,
    TokenFormat,
};

/// TODO: docs.
pub(crate) type MultipartUploadId<Ctx> =
    <<<Ctx as Context>::ObjectGateway as ObjectGateway>::Storage as ObjectStorage>::MultipartUploadId;

/// TODO: docs.
pub(crate) type ObjectStorageError<Ctx> =
    <<<Ctx as Context>::ObjectGateway as ObjectGateway>::Storage as ObjectStorage>::Error;

/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {
    /// The database storing all the caches' metadatas.
    type CacheDb: CacheDb;

    /// The CDN that sits in front of the cache server.
    type Cdn: Cdn;

    /// TODO: docs.
    type Clock: Clock;

    /// The gateway to the object storage that holds the NARs and NARInfos.
    type ObjectGateway: ObjectGateway;

    /// The verifier of the signatures of the OIDC tokens CI jobs exchange for
    /// tokens.
    type OidcVerifier: OidcVerifier;

    /// A cryptographically secure source of randomness.
    type Rng: CryptoRng;

    /// The format of the tokens given to clients.
    type TokenFormat: TokenFormat;

    /// Returns a reference to the cache database.
    fn cache_db(&self) -> &Self::CacheDb;

    /// Returns a reference to the CDN.
    fn cdn(&self) -> &Self::Cdn;

    /// Returns a reference to the clock.
    fn clock(&self) -> &Self::Clock;

    /// Returns a reference to the object gateway.
    fn object_gateway(&self) -> &Self::ObjectGateway;

    /// Returns a reference to the OIDC verifier.
    fn oidc_verifier(&self) -> &Self::OidcVerifier;

    /// Returns the origin the server is reachable at, e.g.
    /// `https://cache.example.com`.
    ///
    /// Clients rely on it, e.g. as the audience of the OIDC tokens they
    /// exchange, so changing it breaks the clients that still use the old one.
    fn origin(&self) -> &str;

    /// Returns a source of randomness.
    fn rng(&self) -> Self::Rng;

    /// Returns a reference to the token format.
    fn token_format(&self) -> &Self::TokenFormat;

    /// Returns a reference to the object storage.
    fn object_storage(
        &self,
    ) -> &<Self::ObjectGateway as ObjectGateway>::Storage {
        self.object_gateway().storage()
    }

    /// TODO: docs.
    fn want_mass_query(&self) -> bool {
        true
    }
}
