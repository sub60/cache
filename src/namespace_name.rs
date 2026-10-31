use core::fmt;
use core::ops::Deref;
use core::str::FromStr;

use smol_str::SmolStr;

/// The type representing the name of a namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamespaceName(SmolStr);

/// The type of error that can occur when parsing a string into a
/// [`NamespaceName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum NamespaceNameFromStrError {
    /// The string was empty.
    #[display("Namespace name cannot be empty")]
    Empty,

    /// The string was longer than the [maximum](NamespaceName::MAX_LEN) length.
    #[display(
        "Namespace name cannot be longer than {} characters",
        NamespaceName::MAX_LEN
    )]
    TooLong,

    /// The namespace name contains an invalid character at the given offset.
    #[display(
        "Namespace name contains an invalid character at offset {offset}: \
         '{character}'"
    )]
    InvalidCharacter {
        /// The invalid character.
        character: char,

        /// The offset of the invalid character.
        offset: usize,
    },
}

impl NamespaceName {
    /// The maximum length of a namespace name, in bytes.
    pub const MAX_LEN: usize = 64;

    fn validate(s: &str) -> Result<(), NamespaceNameFromStrError> {
        if s.is_empty() {
            return Err(NamespaceNameFromStrError::Empty);
        }

        if s.len() > Self::MAX_LEN {
            return Err(NamespaceNameFromStrError::TooLong);
        }

        for (offset, character) in s.char_indices() {
            if !character.is_ascii_alphanumeric()
                && character != '-'
                && character != '_'
            {
                return Err(NamespaceNameFromStrError::InvalidCharacter {
                    character,
                    offset,
                });
            }
        }

        Ok(())
    }
}

impl fmt::Display for NamespaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for NamespaceName {
    type Err = NamespaceNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(Self(SmolStr::new(s)))
    }
}

impl TryFrom<&str> for NamespaceName {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Deref for NamespaceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::de::Deserialize<'de> for NamespaceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = SmolStr::deserialize(deserializer)?;
        Self::validate(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(s))
    }
}
