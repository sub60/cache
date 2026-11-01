//! Verifying JWTs signed by OIDC issuers.

use alloc::borrow::Cow;
use core::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::de::DeserializeOwned;

use crate::{Clock, Context, OidcIssuer, OidcVerifier, Rs256Signature};

/// How far off the issuer's clock can be from ours.
const CLOCK_LEEWAY: Duration = Duration::from_mins(1);

/// The type of error that can occur when verifying a JWT.
#[derive(Debug, derive_more::Display, cauchy::Error)]
pub(crate) enum JwtError<VerifierError> {
    /// The JWT has expired.
    #[display("the token has expired")]
    Expired,

    /// The JWT's audience isn't the expected one.
    #[display("the token's audience must be {_0:?}")]
    InvalidAudience(String),

    /// The JWT's claims don't have the expected shape.
    #[display("the token's claims are invalid: {_0}")]
    InvalidClaims(serde_json::Error),

    /// The JWT's issuer isn't the expected one.
    #[display("the token's issuer must be {_0:?}")]
    InvalidIssuer(&'static str),

    /// The JWT's signature wasn't made with any of the issuer's keys.
    #[display("the token's signature is invalid")]
    InvalidSignature,

    /// The JWT isn't made of a base64url-encoded header, payload and
    /// signature separated by dots.
    #[display("the token is not a valid JWT")]
    Malformed,

    /// The JWT isn't valid yet.
    #[display("the token is not valid yet")]
    NotYetValid,

    /// The JWT is signed with an algorithm other than RS256.
    #[display("the token must be signed with RS256, not {_0:?}")]
    UnsupportedAlgorithm(String),

    /// The signature couldn't be verified.
    #[display("couldn't verify the token's signature: {_0}")]
    Verifier(VerifierError),
}

#[derive(serde::Deserialize)]
struct Header<'a> {
    #[serde(borrow)]
    alg: Cow<'a, str>,
    #[serde(borrow)]
    kid: Cow<'a, str>,
}

#[derive(serde::Deserialize)]
struct RegisteredClaims {
    iss: String,
    aud: Audience,
    exp: u64,
    nbf: Option<u64>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

/// Verifies that the JWT was signed by the issuer for the server's origin, and
/// that it's valid now, then returns its claims.
pub(crate) async fn verify<Claims, Ctx>(
    jwt: &str,
    issuer: OidcIssuer,
    ctx: &Ctx,
) -> Result<Claims, JwtError<<Ctx::OidcVerifier as OidcVerifier>::Error>>
where
    Claims: DeserializeOwned,
    Ctx: Context,
{
    let (message, signature) =
        jwt.rsplit_once('.').ok_or(JwtError::Malformed)?;

    let (header, payload) =
        message.split_once('.').ok_or(JwtError::Malformed)?;

    if payload.contains('.') {
        return Err(JwtError::Malformed);
    }

    let header_bytes = decode(header)?;
    let header = serde_json::from_slice::<Header<'_>>(&header_bytes)
        .map_err(|_| JwtError::Malformed)?;

    // Only accepting RS256 means a token can't pick a weaker algorithm, or
    // `none`, for itself.
    if header.alg != "RS256" {
        return Err(JwtError::UnsupportedAlgorithm(header.alg.into_owned()));
    }

    let signature = decode(signature)?;

    let is_valid = ctx
        .oidc_verifier()
        .verify_rs256(
            issuer,
            Rs256Signature {
                key_id: &header.kid,
                message: message.as_bytes(),
                signature: &signature,
            },
        )
        .await
        .map_err(JwtError::Verifier)?;

    if !is_valid {
        return Err(JwtError::InvalidSignature);
    }

    let payload = decode(payload)?;

    let claims = serde_json::from_slice::<RegisteredClaims>(&payload)
        .map_err(JwtError::InvalidClaims)?;

    if claims.iss != issuer.url() {
        return Err(JwtError::InvalidIssuer(issuer.url()));
    }

    let audience = ctx.origin();

    let has_audience = match &claims.aud {
        Audience::One(aud) => aud == audience,
        Audience::Many(auds) => auds.iter().any(|aud| aud == audience),
    };

    if !has_audience {
        return Err(JwtError::InvalidAudience(audience.to_owned()));
    }

    let now = ctx.clock().unix_ts();

    if Duration::from_secs(claims.exp) + CLOCK_LEEWAY <= now {
        return Err(JwtError::Expired);
    }

    if let Some(nbf) = claims.nbf
        && now + CLOCK_LEEWAY < Duration::from_secs(nbf)
    {
        return Err(JwtError::NotYetValid);
    }

    serde_json::from_slice(&payload).map_err(JwtError::InvalidClaims)
}

fn decode<VerifierError>(
    part: &str,
) -> Result<Vec<u8>, JwtError<VerifierError>> {
    URL_SAFE_NO_PAD.decode(part).map_err(|_| JwtError::Malformed)
}
