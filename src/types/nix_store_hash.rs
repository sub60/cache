use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

/// The type representing the 32-character hash of a Nix store path.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NixStoreHash([u8; Self::LEN]);

impl NixStoreHash {
    /// Nix's custom base32 alphabet for store-path hashes, which includes all
    /// digits and all lowercase letters except 'e', 'o', 't', and 'u'.
    pub const ALPHABET: &[u8; 32] = b"0123456789abcdfghijklmnpqrsvwxyz";

    /// The length of a Nix store hash, in bytes.
    pub const LEN: usize = 32;

    /// Returns the hash as a string slice.
    pub fn as_str(&self) -> &str {
        // SAFETY: the only way to create instances of NixStoreHash is through
        // its FromStr implementation, which validates the string.
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }

    const fn is_byte_valid(byte: u8) -> bool {
        const VALID_BYTES: [bool; 256] = {
            let mut lut = [false; 256];
            let mut idx = 0;

            while idx < NixStoreHash::ALPHABET.len() {
                lut[NixStoreHash::ALPHABET[idx] as usize] = true;
                idx += 1;
            }

            lut
        };

        VALID_BYTES[byte as usize]
    }

    fn validate(s: &str) -> Result<(), NixStoreHashFromStrError> {
        if s.len() != Self::LEN {
            return Err(NixStoreHashFromStrError::InvalidLength);
        }

        for (offset, byte) in s.bytes().enumerate() {
            if !Self::is_byte_valid(byte) {
                let character = s[offset..]
                    .chars()
                    .next()
                    .expect("offset must be less that string length");
                return Err(NixStoreHashFromStrError::InvalidCharacter {
                    character,
                    offset,
                });
            }
        }

        Ok(())
    }
}

/// The type of error that can occur when parsing a string into a
/// [`NixStoreHash`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum NixStoreHashFromStrError {
    /// The string was not exactly 32 characters long.
    #[display(
        "Nix store hash must be exactly {} characters long",
        NixStoreHash::LEN
    )]
    InvalidLength,

    /// The hash contains an invalid character at the given offset.
    #[display(
        "Nix store hash contains an invalid character at offset {offset}: \
         '{character}'"
    )]
    InvalidCharacter {
        /// The invalid character.
        character: char,

        /// The offset of the first instance of the invalid character in the
        /// string.
        offset: usize,
    },
}

impl fmt::Display for NixStoreHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for NixStoreHash {
    type Err = NixStoreHashFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        let bytes = s.as_bytes().try_into().expect("string has been validated");
        Ok(Self(bytes))
    }
}

impl<'de> serde::de::Deserialize<'de> for NixStoreHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = Cow::<'de, str>::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}
