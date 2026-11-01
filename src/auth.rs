//! Types related to the `/-/api/v1/auth` endpoints.

use core::fmt;
use core::str::FromStr;
use core::time::Duration;

/// The body of a `/-/api/v1/auth/github-actions` request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GitHubActionsAuthRequestBody {
    /// The OIDC token the job got from GitHub, requested with the server's
    /// origin as its audience.
    pub token: String,
}

/// The body of a successful response from an auth endpoint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuthResponseBody {
    /// The token to authenticate requests with, as a bearer token in their
    /// `Authorization` header.
    pub token: String,

    /// When the token stops being valid, as the time since the UNIX epoch.
    #[serde(with = "crate::serde_utils::secs")]
    pub expires_at: Duration,
}

macro_rules! impl_json_display_and_from_str {
    ($ty:ty) => {
        impl fmt::Display for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let json =
                    serde_json::to_string(self).map_err(|_| fmt::Error)?;
                f.write_str(&json)
            }
        }

        impl FromStr for $ty {
            type Err = serde_json::Error;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                serde_json::from_str(s)
            }
        }
    };
}

impl_json_display_and_from_str!(GitHubActionsAuthRequestBody);
impl_json_display_and_from_str!(AuthResponseBody);
