use alloc::borrow::Cow;
use core::fmt;
use core::str::FromStr;

type Md5Digest = [u8; 16];

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Etag(Md5Digest);

/// The type of error that can occur when parsing a string into an [`Etag`].
#[derive(Debug, Copy, Clone, derive_more::Display, cauchy::Error)]
pub enum EtagFromStrError {
    /// The string's format is not a 32-character hexadecimal MD5 digest.
    #[display("Etag must be a 32-character hexadecimal MD5 digest")]
    InvalidFormat,

    /// The string contains a non-hexadecimal character.
    #[display("Etag contains a non-hexadecimal character")]
    InvalidHex,
}

impl fmt::Display for Etag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
    }
}

impl FromStr for Etag {
    type Err = EtagFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn decode_nibble(byte: u8) -> Result<u8, EtagFromStrError> {
            match byte {
                b'0'..=b'9' => Ok(byte - b'0'),
                b'a'..=b'f' => Ok(byte - b'a' + 10),
                b'A'..=b'F' => Ok(byte - b'A' + 10),
                _ => Err(EtagFromStrError::InvalidHex),
            }
        }

        let bytes = s.as_bytes();

        if bytes.len() != 32 {
            return Err(EtagFromStrError::InvalidFormat);
        }

        let mut digest = [0; 16];

        for (idx, byte) in digest.iter_mut().enumerate() {
            let high = decode_nibble(bytes[idx * 2])?;
            let low = decode_nibble(bytes[idx * 2 + 1])?;
            *byte = high << 4 | low;
        }

        Ok(Self(digest))
    }
}

impl serde::ser::Serialize for Etag {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::de::Deserialize<'de> for Etag {
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
        let etag = "d41d8cd98f00b204e9800998ecf8427e".parse::<Etag>().unwrap();
        assert_eq!(etag.to_string(), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(etag.to_string().parse::<Etag>().unwrap(), etag);
    }
}
