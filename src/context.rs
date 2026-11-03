//! TODO: docs.

use rand_core::CryptoRng;

use crate::{
    CacheDb,
    Cdn,
    Clock,
    NarInfoStorage,
    NarStorage,
    OidcVerifier,
    TokenFormat,
};

/// TODO: docs.
pub(crate) type MultipartUploadId<Ctx> =
    <<Ctx as Context>::NarStorage as NarStorage>::MultipartUploadId;

/// The type of error returned by the context's [`NarStorage`].
pub(crate) type NarStorageError<Ctx> =
    <<Ctx as Context>::NarStorage as NarStorage>::Error;

/// The type of error returned by the context's [`NarInfoStorage`].
pub(crate) type NarInfoStorageError<Ctx> =
    <<Ctx as Context>::NarInfoStorage as NarInfoStorage>::Error;

/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {
    /// The database storing all the caches' metadatas.
    type CacheDb: CacheDb;

    /// The CDN that sits in front of the cache server.
    type Cdn: Cdn;

    /// TODO: docs.
    type Clock: Clock;

    /// The storage of the caches' NARInfos.
    type NarInfoStorage: NarInfoStorage;

    /// The storage of the caches' NARs.
    type NarStorage: NarStorage;

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

    /// Returns a reference to the NAR storage.
    fn nar_storage(&self) -> &Self::NarStorage;

    /// Returns a reference to the NARInfo storage.
    fn narinfo_storage(&self) -> &Self::NarInfoStorage;

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

    /// TODO: docs.
    fn want_mass_query(&self) -> bool {
        true
    }
}
