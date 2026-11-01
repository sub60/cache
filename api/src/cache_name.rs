use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;
use core::str::FromStr;

use smol_str::SmolStr;

/// The type representing the name of a cache.
///
/// Valid names are between 1 and [`MAX_LEN`](Self::MAX_LEN) bytes long, and can
/// only contain ASCII letters and digits, `-`, `_` and `.`. Any of them can be
/// the first character, but `.` and `..` are not valid names.
///
/// Names keep the casing they were created with (e.g. `Foo`). Two names that
/// only differ in casing are considered equal as far as [`PartialEq`] and
/// [`Hash`] are concerned.
#[derive(Debug, Clone)]
pub struct CacheName(SmolStr);

/// The type of error that can occur when parsing a string into a
/// [`CacheName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum CacheNameFromStrError {
    /// The string was empty.
    #[display("Cache name cannot be empty")]
    Empty,

    /// The string was longer than the [maximum](CacheName::MAX_LEN) length.
    #[display(
        "Cache name cannot be longer than {} characters",
        CacheName::MAX_LEN
    )]
    TooLong,

    /// The string was `.` or `..`.
    #[display("Cache name cannot be '.' or '..'")]
    DotOrDotDot,

    /// The cache name contains an invalid character at the given offset.
    #[display(
        "Cache name contains an invalid character at offset {offset}: \
         '{character}'"
    )]
    InvalidCharacter {
        /// The invalid character.
        character: char,

        /// The offset of the invalid character.
        offset: usize,
    },
}

impl CacheName {
    /// The maximum length of a cache name, in bytes.
    pub const MAX_LEN: usize = 100;

    fn validate(s: &str) -> Result<(), CacheNameFromStrError> {
        if s.is_empty() {
            return Err(CacheNameFromStrError::Empty);
        }

        if s.len() > Self::MAX_LEN {
            return Err(CacheNameFromStrError::TooLong);
        }

        // These would be resolved as relative path segments in URLs.
        if s == "." || s == ".." {
            return Err(CacheNameFromStrError::DotOrDotDot);
        }

        for (offset, character) in s.char_indices() {
            if !character.is_ascii_alphanumeric()
                && character != '-'
                && character != '_'
                && character != '.'
            {
                return Err(CacheNameFromStrError::InvalidCharacter {
                    character,
                    offset,
                });
            }
        }

        Ok(())
    }
}

impl PartialEq for CacheName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for CacheName {}

impl Hash for CacheName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Must be consistent with `PartialEq`, so names that only differ in
        // casing must hash the same.
        for byte in self.0.bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
        // Delimit the name like `str`'s `Hash` impl does, so that hashing a
        // tuple of names doesn't depend on where one ends and the next starts.
        state.write_u8(0xff);
    }
}

impl fmt::Display for CacheName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for CacheName {
    type Err = CacheNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(Self(SmolStr::new(s)))
    }
}

impl TryFrom<&str> for CacheName {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Deref for CacheName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::de::Deserialize<'de> for CacheName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = SmolStr::deserialize(deserializer)?;
        Self::validate(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(s))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hash::DefaultHasher;

    use super::*;

    fn name(s: &str) -> CacheName {
        s.parse().unwrap()
    }

    fn hash_of(value: &impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn accepts_valid_names() {
        let longest = "a".repeat(CacheName::MAX_LEN);
        for s in ["foo", ".foo", "foo.bar", "-", "_", "-foo", "_foo", &longest]
        {
            assert!(s.parse::<CacheName>().is_ok(), "{s}");
        }
    }

    #[test]
    fn rejects_dot_and_dot_dot() {
        for s in [".", ".."] {
            assert_eq!(
                s.parse::<CacheName>().unwrap_err(),
                CacheNameFromStrError::DotOrDotDot,
                "{s}",
            );
        }
    }

    #[test]
    fn rejects_invalid_names() {
        assert_eq!(
            "".parse::<CacheName>().unwrap_err(),
            CacheNameFromStrError::Empty,
        );
        assert_eq!(
            "a".repeat(CacheName::MAX_LEN + 1)
                .parse::<CacheName>()
                .unwrap_err(),
            CacheNameFromStrError::TooLong,
        );
        assert_eq!(
            "foo/bar".parse::<CacheName>().unwrap_err(),
            CacheNameFromStrError::InvalidCharacter {
                character: '/',
                offset: 3
            },
        );
    }

    #[test]
    fn keeps_casing() {
        assert_eq!(name("Foo").to_string(), "Foo");
    }

    #[test]
    fn eq_ignores_case() {
        assert_eq!(name("Foo"), name("foo"));
        assert_eq!(name("Foo"), name("FOO"));
        assert_ne!(name("Foo"), name("foo-2"));
    }

    #[test]
    fn hash_ignores_case() {
        let names = HashSet::from([name("Foo")]);
        assert!(names.contains(&name("foo")));
        assert!(!names.contains(&name("foo-2")));
    }

    #[test]
    fn hash_delimits_names() {
        assert_ne!(
            hash_of(&(name("ab"), name("c"))),
            hash_of(&(name("a"), name("bc"))),
        );
    }
}
