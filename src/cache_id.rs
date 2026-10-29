use core::fmt;
use core::str::FromStr;
use core::time::Duration;

use ulid::Ulid;

/// The type representing the stable ID of a cache.
///
/// This must never change for the lifetime of the cache, and must never be
/// reused for a different cache.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CacheId(Ulid);

/// The type of error that can occur when parsing a string into a [`CacheId`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum CacheIdFromStrError {
    /// The string didn't have the right length.
    #[display("Cache ID must be {} characters long", ulid::ULID_LEN)]
    InvalidLength,

    /// The string contained an invalid character.
    #[display("Cache ID contains an invalid character")]
    InvalidCharacter,
}

impl CacheId {
    /// Creates the ID of a cache created at the given time since the UNIX
    /// epoch, using the given random bytes to make it unique.
    ///
    /// Only the millisecond precision of the creation time is preserved.
    #[inline]
    pub fn new(created_at: Duration, random: [u8; 10]) -> Self {
        let mut random_u128 = [0; 16];
        random_u128[6..].copy_from_slice(&random);
        Self(Ulid::from_parts(
            created_at.as_millis() as u64,
            u128::from_be_bytes(random_u128),
        ))
    }
}

impl FromStr for CacheId {
    type Err = CacheIdFromStrError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ulid::from_string(s).map(Self).map_err(|err| match err {
            ulid::DecodeError::InvalidLength => {
                CacheIdFromStrError::InvalidLength
            },
            ulid::DecodeError::InvalidChar => {
                CacheIdFromStrError::InvalidCharacter
            },
        })
    }
}

impl fmt::Display for CacheId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0; ulid::ULID_LEN];
        let encoded = self.0.array_to_str(&mut buf);
        encoded.make_ascii_lowercase();
        f.write_str(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id() -> CacheId {
        CacheId::new(Duration::from_secs(1_700_000_000), [42; 10])
    }

    #[test]
    fn display_is_lowercase() {
        let id_str = test_id().to_string();
        assert_eq!(id_str, id_str.to_ascii_lowercase());
        assert_eq!(id_str.len(), 26);
    }

    #[test]
    fn display_from_str_roundtrip() {
        let id = test_id();
        assert_eq!(id.to_string().parse::<CacheId>().unwrap(), id);
    }

    #[test]
    fn from_str_is_case_insensitive() {
        let lower = "01jq7gk8ar0000000000000016".parse::<CacheId>().unwrap();
        let upper = "01JQ7GK8AR0000000000000016".parse::<CacheId>().unwrap();
        assert_eq!(lower, upper);
    }
}
