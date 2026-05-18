use core::fmt;
use core::ops::Deref;
use core::str::FromStr;

use smol_str::SmolStr;

/// The type representing the name of a user.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserName(SmolStr);

/// The type of error that can occur when parsing a string into a [`UserName`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum UserNameFromStrError {
    /// The string was empty.
    #[display("Username cannot be empty")]
    Empty,

    /// The string was longer than the [maximum](UserName::MAX_LEN) length.
    #[display(
        "Username cannot be longer than {} characters",
        UserName::MAX_LEN
    )]
    TooLong,

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

impl UserName {
    /// The maximum length of a username, in bytes.
    pub const MAX_LEN: usize = 64;

    fn validate(s: &str) -> Result<(), UserNameFromStrError> {
        if s.is_empty() {
            return Err(UserNameFromStrError::Empty);
        }

        if s.len() > Self::MAX_LEN {
            return Err(UserNameFromStrError::TooLong);
        }

        for (offset, character) in s.char_indices() {
            if !character.is_ascii_alphanumeric()
                && character != '-'
                && character != '_'
            {
                return Err(UserNameFromStrError::InvalidCharacter {
                    character,
                    offset,
                });
            }
        }

        Ok(())
    }
}

impl fmt::Display for UserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for UserName {
    type Err = UserNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(Self(SmolStr::new(s)))
    }
}

impl TryFrom<&str> for UserName {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Deref for UserName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::de::Deserialize<'de> for UserName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = SmolStr::deserialize(deserializer)?;
        Self::validate(&s).map_err(serde::de::Error::custom)?;
        Ok(Self(s))
    }
}
