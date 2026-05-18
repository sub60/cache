use core::fmt;
use core::str::FromStr;

use smol_str::SmolStr;

/// The type representing the name of a derivation in the Nix store.
///
/// This is the part of a store path that comes after the hash and a dash, e.g
/// `"hello"` in `"/nix/store/0123456789abcdfghijklmnpqrsvwxyz-hello"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StoreName(SmolStr);

impl StoreName {
    const MAX_LEN: usize = 211;

    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    const fn is_byte_valid(byte: u8) -> bool {
        matches!(byte, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'+' | b'-' | b'.' | b'_' | b'?' | b'=')
    }

    fn validate(s: &str) -> Result<(), StoreNameFromStrError> {
        if s.is_empty() {
            return Err(StoreNameFromStrError::Empty);
        }

        if s.len() > Self::MAX_LEN {
            return Err(StoreNameFromStrError::TooLong);
        }

        for (offset, byte) in s.bytes().enumerate() {
            if !Self::is_byte_valid(byte) {
                let character = s[offset..]
                    .chars()
                    .next()
                    .expect("offset must be less that string length");
                return Err(StoreNameFromStrError::InvalidCharacter {
                    character,
                    offset,
                });
            }
        }

        let bytes = s.as_bytes();

        if bytes[0] == b'.' {
            let Some(&second_byte) = bytes.get(1) else {
                return Err(StoreNameFromStrError::StringIsDot);
            };
            if second_byte == b'-' {
                return Err(
                    StoreNameFromStrError::FirstDashSeparatedComponentIsDot,
                );
            }
            if second_byte == b'.' {
                let Some(&third_byte) = bytes.get(2) else {
                    return Err(StoreNameFromStrError::StringIsDoubleDot);
                };
                if third_byte == b'-' {
                    return Err(
                        StoreNameFromStrError::FirstDashSeparatedComponentIsDoubleDot,
                    );
                }
            }
        }

        Ok(())
    }
}

/// The type of error that can occur when parsing a string into a [`StoreName`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum StoreNameFromStrError {
    /// The string was empty.
    #[display("Nix store name can't be empty")]
    Empty,

    /// The string was longer than 211 characters.
    #[display(
        "Nix store name can't be longer than {} characters",
        StoreName::MAX_LEN
    )]
    TooLong,

    /// The name contains an invalid character at the given offset.
    #[display(
        "Nix store name contains an invalid character at offset {offset}: \
         '{character}'"
    )]
    InvalidCharacter {
        /// The invalid character.
        character: char,

        /// The offset of the first instance of the invalid character in the
        /// string.
        offset: usize,
    },

    /// The string was a `.`, which is not a valid name.
    #[display("Nix store name can't be a dot ('.')")]
    StringIsDot,

    /// The string was a `..`, which is not a valid name.
    #[display("Nix store name can't be a double dot ('..')")]
    StringIsDoubleDot,

    /// The part of the string before the first dash is a `.`, which is not
    /// allowed.
    #[display(
        "The part of the name before the first dash can't be a dot ('.')"
    )]
    FirstDashSeparatedComponentIsDot,

    /// The part of the string before the first dash is a `..`, which is not
    /// allowed.
    #[display(
        "The part of the name before the first dash can't be a double dot \
         ('..')"
    )]
    FirstDashSeparatedComponentIsDoubleDot,
}

impl fmt::Display for StoreName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for StoreName {
    type Err = StoreNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(Self(SmolStr::new(s)))
    }
}

impl<'de> serde::de::Deserialize<'de> for StoreName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = SmolStr::deserialize(deserializer)?;
        Self::validate(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(s))
    }
}
