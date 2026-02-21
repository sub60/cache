use core::fmt;

use smol_str::SmolStr;

use crate::types::WithRef;

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

impl fmt::Display for NixKeyOrSignatureName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
