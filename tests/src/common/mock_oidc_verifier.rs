use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use sub60_cache::{OidcIssuer, OidcVerifier, Rs256Signature};

use crate::common::Never;

/// An [`OidcVerifier`] that accepts the signatures made by
/// [`sign`](Self::sign), so tests can create OIDC tokens without RSA keys.
///
/// A signature is the SHA-256 hash of the key ID followed by the message.
#[derive(Copy, Clone, Default)]
pub struct MockOidcVerifier;

impl MockOidcVerifier {
    /// The ID of the only key the issuers have.
    pub const KEY_ID: &str = "test-key";

    /// Returns a JWT with the given header and claims, signed with the key
    /// with the given ID.
    pub fn sign(
        header: &serde_json::Value,
        claims: &serde_json::Value,
        key_id: &str,
    ) -> String {
        let message = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(header.to_string()),
            URL_SAFE_NO_PAD.encode(claims.to_string()),
        );

        let signature = Self::signature(key_id, message.as_bytes());

        format!("{message}.{}", URL_SAFE_NO_PAD.encode(signature))
    }

    fn signature(key_id: &str, message: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(key_id);
        hasher.update(message);
        hasher.finalize().into()
    }
}

impl OidcVerifier for MockOidcVerifier {
    type Error = Never;

    async fn verify_rs256(
        &self,
        _: OidcIssuer,
        signature: Rs256Signature<'_>,
    ) -> Result<bool, Self::Error> {
        Ok(signature.key_id == Self::KEY_ID
            && signature.signature
                == Self::signature(signature.key_id, signature.message))
    }
}
