//! Verifying the signatures of OIDC tokens.

use core::fmt;

use axum::http;

/// A trait for verifying that OIDC tokens were signed by their issuer.
pub trait OidcVerifier {
    /// The type of error that can occur when a signature can't be checked,
    /// e.g. because the issuer's keys couldn't be fetched.
    type Error: fmt::Display + Into<http::StatusCode>;

    /// Returns whether the signature was made with the issuer's key with the
    /// given ID.
    ///
    /// The issuer's keys are the JWK set served at its
    /// [`jwks_url`](OidcIssuer::jwks_url). Implementations should cache them,
    /// and fetch them again when asked for a key ID that isn't cached, since
    /// that's how issuers rotate keys.
    ///
    /// Anything wrong with the signature itself, including an unknown key ID or
    /// bytes that aren't a valid signature, must return `false`, not an error.
    /// Errors are for when the signature couldn't be checked.
    fn verify_rs256(
        &self,
        issuer: OidcIssuer,
        signature: Rs256Signature<'_>,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}

/// An issuer of OIDC tokens.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OidcIssuer {
    /// The issuer of the tokens GitHub Actions jobs on `github.com` can
    /// request.
    GitHubActions,
}

/// An RS256 signature of a JWT.
#[derive(Debug, Copy, Clone)]
pub struct Rs256Signature<'a> {
    /// The ID of the issuer's key the JWT says it was signed with, i.e. the
    /// `kid` in its header.
    pub key_id: &'a str,

    /// What was signed, i.e. the JWT's header and payload as they appear in
    /// the JWT, joined by a `.`.
    pub message: &'a [u8],

    /// The signature's bytes.
    pub signature: &'a [u8],
}

impl OidcIssuer {
    /// Returns the URL of the issuer's JWK set.
    #[inline]
    pub fn jwks_url(self) -> &'static str {
        match self {
            Self::GitHubActions => {
                "https://token.actions.githubusercontent.com/.well-known/jwks"
            },
        }
    }

    /// Returns the issuer's URL, i.e. the `iss` claim of the tokens it signs.
    #[inline]
    pub fn url(self) -> &'static str {
        match self {
            Self::GitHubActions => {
                "https://token.actions.githubusercontent.com"
            },
        }
    }
}
