use core::fmt;
use core::str::FromStr;

use base64::Engine;
use smol_str::SmolStr;

use crate::WithRef;

/// TODO: docs.
pub type CacheSigningKey = NixKeyOrSignature<ed25519_dalek::SigningKey>;

/// TODO: docs.
pub type CachePublicKey = NixKeyOrSignature<ed25519_dalek::VerifyingKey>;

/// TODO: docs.
pub type NixSignature = NixKeyOrSignature<ed25519_dalek::Signature>;

/// TODO: docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixKeyOrSignature<KeyOrSig> {
    name: NixKeyOrSignatureName,
    key_or_sig: KeyOrSig,
}

/// The type representing the name of a key or signature, like
/// "cache.example.org-1".
///
/// This is a thin wrapper around the inner string type, whose [`FromStr`]
/// implementation validates that the string is not empty and doesn't contain
/// the `:` character, which is used to separate the name from the key or
/// signature in the [`NixKeyOrSignature`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixKeyOrSignatureName(SmolStr);

/// The type of error that can occur when parsing a string into a
/// [`NixKeyOrSignatureName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NixKeyOrSignatureFromStrError<KeyOrSignatureFromBytesError> {
    /// The key or signature was not valid base64-encoded data.
    #[display("could not decode base64-encoded key or signature: {_0}")]
    DecodeError(base64::DecodeError),

    /// The string did not contain a `:` character separating the name from the
    /// key or signature.
    #[display("missing ':' separating name from key or signature")]
    MissingColon,

    /// There was an error converting the key or signature bytes into the
    /// appropriate type.
    #[display("invalid key/signature: {_0}")]
    InvalidKeyOrSignature(KeyOrSignatureFromBytesError),

    /// The name did not parse into a valid key or signature name.
    #[display("invalid name: {_0}")]
    InvalidName(<NixKeyOrSignatureName as FromStr>::Err),
}

/// The type of error that can occur when parsing a string into a
/// [`NixKeyOrSignatureName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NixKeyOrSignatureNameFromStrError {
    /// The string was empty.
    #[display("the name of a key or signature cannot be empty")]
    Empty,

    /// The string contained a `:` character, which is not allowed in a
    /// [`NixKeyOrSignatureName`] because it is used to separate the name from
    /// the key or signature in the [`NixKeyOrSignature`].
    #[display("the name of a key or signature cannot contain a ':' character")]
    ContainsColon,
}

impl CacheSigningKey {
    /// Creates a new random [`CacheSigningKey`] with the given name from the
    /// given RNG.
    pub fn from_rng(
        name: NixKeyOrSignatureName,
        rng: &mut impl rand_core::Rng,
    ) -> Self {
        let mut bytes = [0u8; _];
        rng.fill_bytes(&mut bytes);
        Self::new(name, bytes)
    }

    /// Creates a new [`CacheSigningKey`] with the given name from the given
    /// bytes.
    pub fn new(name: NixKeyOrSignatureName, bytes: [u8; 32]) -> Self {
        Self { name, key_or_sig: ed25519_dalek::SigningKey::from_bytes(&bytes) }
    }

    /// Converts the signing key into the corresponding public key which users
    /// can add to the `trusted-public-keys` list in their Nix configuration.
    pub fn public_key(&self) -> CachePublicKey {
        CachePublicKey {
            name: self.name.clone(),
            key_or_sig: self.key_or_sig.verifying_key(),
        }
    }

    /// Signs the given data, producing a [`NixSignature`] with the same
    /// [name](NixKeyOrSignatureName) as this signing key.
    pub fn sign(&self, data: &[u8]) -> NixSignature {
        use ed25519_dalek::Signer;
        NixSignature {
            name: self.name.clone(),
            key_or_sig: self.key_or_sig.sign(data),
        }
    }
}

impl NixKeyOrSignatureName {
    /// Returns the name as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<KeyOrSig: WithRef<[u8]>> fmt::Display for NixKeyOrSignature<KeyOrSig> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key_or_sig.with_ref(|key_or_sig_bytes| {
            let base64_bytes = base64::display::Base64Display::new(
                key_or_sig_bytes,
                &base64::engine::general_purpose::STANDARD,
            );
            write!(f, "{}:{}", self.name, base64_bytes)
        })
    }
}

impl<KeyOrSig, E> FromStr for NixKeyOrSignature<KeyOrSig>
where
    KeyOrSig: for<'a> TryFrom<&'a [u8], Error = E>,
{
    type Err = NixKeyOrSignatureFromStrError<E>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name_str, key_or_sig_str) = s
            .split_once(':')
            .ok_or(NixKeyOrSignatureFromStrError::MissingColon)?;

        let name = name_str
            .parse()
            .map_err(NixKeyOrSignatureFromStrError::InvalidName)?;

        let key_or_sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(key_or_sig_str)
            .map_err(NixKeyOrSignatureFromStrError::DecodeError)?;

        let key_or_sig = KeyOrSig::try_from(key_or_sig_bytes.as_slice())
            .map_err(NixKeyOrSignatureFromStrError::InvalidKeyOrSignature)?;

        Ok(Self { name, key_or_sig })
    }
}

impl fmt::Display for NixKeyOrSignatureName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for NixKeyOrSignatureName {
    type Err = NixKeyOrSignatureNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(NixKeyOrSignatureNameFromStrError::Empty);
        }

        if s.contains(':') {
            return Err(NixKeyOrSignatureNameFromStrError::ContainsColon);
        }

        Ok(Self(SmolStr::new(s)))
    }
}

impl PartialEq<str> for NixKeyOrSignatureName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for NixKeyOrSignatureName {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

#[cfg(test)]
mod tests {
    use crate::CacheSigningKey;

    #[test]
    fn cache_signing_key_from_str() {
        let key = "cache.sub60.dev/noib3/dotfiles-1:\
                   +SYP16zDBSBu7o+W+pdjSURr6KKFql0y2UXnYOPsqpk=";
        let parsed = key.parse::<CacheSigningKey>().unwrap();
        assert_eq!(parsed.name, "cache.sub60.dev/noib3/dotfiles-1");
    }
}
