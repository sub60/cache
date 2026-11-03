use core::convert::Infallible;
use core::time::Duration;

use axum::http::{HeaderValue, header};
use axum::response::{IntoResponseParts, ResponseParts};
use compact_str::format_compact;

use crate::{Permission, Permissions};

/// The [`Header`] of responses that must never be stored.
const NO_STORE: Header = Header(HeaderValue::from_static("no-store"));

/// Controls how long the shared caches between the clients and the server, like
/// a CDN, can store the responses of public caches.
pub trait CacheControl {
    /// Returns how long shared caches can store the NARs of public caches.
    fn nar_max_age(&self) -> Duration {
        Duration::from_hours(365 * 24)
    }

    /// Returns how long shared caches can store the NARInfos of public caches.
    fn narinfo_max_age(&self) -> Duration {
        Duration::from_hours(24)
    }

    /// Returns how long shared caches can store the `nix-cache-info` of public
    /// caches.
    fn nix_cache_info_max_age(&self) -> Duration {
        Duration::from_hours(1)
    }

    /// Returns the `Cache-Control` header of a NAR of a cache with the given
    /// public permissions.
    #[doc(hidden)]
    fn for_nar(&self, public_permissions: Permissions) -> Header {
        // NARs are content-addressed, so they never change.
        Header::new(self.nar_max_age(), true, public_permissions)
    }

    /// Returns the `Cache-Control` header of a NARInfo of a cache with the
    /// given public permissions.
    #[doc(hidden)]
    fn for_narinfo(&self, public_permissions: Permissions) -> Header {
        Header::new(self.narinfo_max_age(), false, public_permissions)
    }

    /// Returns the `Cache-Control` header of the `nix-cache-info` of a cache
    /// with the given public permissions.
    #[doc(hidden)]
    fn for_nix_cache_info(&self, public_permissions: Permissions) -> Header {
        Header::new(self.nix_cache_info_max_age(), false, public_permissions)
    }

    /// Returns the `Cache-Control` header of a response that can change at any
    /// moment, e.g. a 404, which must never be stored.
    #[doc(hidden)]
    fn no_store(&self) -> Header {
        NO_STORE
    }
}

/// A `Cache-Control` header.
#[doc(hidden)]
pub struct Header(HeaderValue);

impl Header {
    /// Returns the header of a response that doesn't change for the given
    /// duration, or ever if it's `immutable`, about a cache with the given
    /// public permissions.
    ///
    /// Only the responses of public caches can be stored. Shared caches key
    /// responses by URL, so they would serve the ones of private caches to
    /// anyone.
    fn new(
        max_age: Duration,
        immutable: bool,
        public_permissions: Permissions,
    ) -> Self {
        if !public_permissions.contains(Permission::PullFromCache) {
            return Self(HeaderValue::from_static("private"));
        }

        let value = format_compact!(
            "public, max-age={}{}",
            max_age.as_secs(),
            if immutable { ", immutable" } else { "" }
        );

        Self(HeaderValue::from_str(&value).expect("valid header value"))
    }
}

impl IntoResponseParts for Header {
    type Error = Infallible;

    fn into_response_parts(
        self,
        mut parts: ResponseParts,
    ) -> Result<ResponseParts, Self::Error> {
        parts.headers_mut().insert(header::CACHE_CONTROL, self.0);
        Ok(parts)
    }
}
