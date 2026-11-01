use alloc::borrow::Cow;
use core::fmt;
use core::str::FromStr;

/// The hash identifying a Git commit.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CommitSha {
    /// A SHA-1 hash, used by most repositories.
    Sha1([u8; 20]),

    /// A SHA-256 hash, used by repositories created with
    /// `--object-format=sha256`.
    Sha256([u8; 32]),
}

/// The type of error that can occur when parsing a string into a
/// [`CommitSha`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum CommitShaFromStrError {
    /// The string is neither 40 nor 64 characters long.
    #[display("commit SHA must be 40 or 64 hexadecimal characters long")]
    InvalidLength,

    /// The string contains a non-hexadecimal character.
    #[display("commit SHA contains a non-hexadecimal character")]
    InvalidHex,
}

impl CommitSha {
    /// Returns the hash's bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Sha1(bytes) => bytes,
            Self::Sha256(bytes) => bytes,
        }
    }
}

impl fmt::Display for CommitSha {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_bytes().iter().try_for_each(|byte| write!(f, "{byte:02x}"))
    }
}

impl FromStr for CommitSha {
    type Err = CommitShaFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn decode<const N: usize>(
            hex: &[u8],
        ) -> Result<[u8; N], CommitShaFromStrError> {
            fn decode_nibble(byte: u8) -> Result<u8, CommitShaFromStrError> {
                match byte {
                    b'0'..=b'9' => Ok(byte - b'0'),
                    b'a'..=b'f' => Ok(byte - b'a' + 10),
                    b'A'..=b'F' => Ok(byte - b'A' + 10),
                    _ => Err(CommitShaFromStrError::InvalidHex),
                }
            }

            let mut bytes = [0; N];

            for (idx, byte) in bytes.iter_mut().enumerate() {
                let high = decode_nibble(hex[idx * 2])?;
                let low = decode_nibble(hex[idx * 2 + 1])?;
                *byte = high << 4 | low;
            }

            Ok(bytes)
        }

        let hex = s.as_bytes();

        match hex.len() {
            40 => decode(hex).map(Self::Sha1),
            64 => decode(hex).map(Self::Sha256),
            _ => Err(CommitShaFromStrError::InvalidLength),
        }
    }
}

impl serde::ser::Serialize for CommitSha {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::de::Deserialize<'de> for CommitSha {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Cow::<'de, str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_from_str_roundtrip() {
        for hex in [
            "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
            "1f2e8b0c3a6d9e4f7b5c2a1d8e3f6b9c0d4e7a2b5c8f1e4d7a0b3c6e9f2a5d8b",
        ] {
            let sha = hex.parse::<CommitSha>().unwrap();
            assert_eq!(sha.to_string(), hex);
        }
    }

    #[test]
    fn from_str_lowercases() {
        let sha = "7FD1A60B01F91B314F59955A4E4D4E80D8EDF11D"
            .parse::<CommitSha>()
            .unwrap();
        assert_eq!(sha.to_string(), "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");
    }

    #[test]
    fn from_str_rejects_invalid_shas() {
        assert_eq!(
            "7fd1a60b".parse::<CommitSha>(),
            Err(CommitShaFromStrError::InvalidLength)
        );
        assert_eq!(
            "7fd1a60b01f91b314f59955a4e4d4e80d8edf11g".parse::<CommitSha>(),
            Err(CommitShaFromStrError::InvalidHex)
        );
    }
}
