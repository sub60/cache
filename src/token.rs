//! Tokens that clients authenticate their requests with.

use core::fmt;

use rand_core::CryptoRng;
use sha2::{Digest, Sha256};

/// A trait for converting [`Token`]s to and from the text clients send.
pub trait TokenFormat {
    /// Formats the token as the text clients send back.
    fn display(&self, token: &Token) -> impl fmt::Display;

    /// The inverse of [`display`](Self::display).
    ///
    /// Returns `None` if the text isn't a token's.
    fn parse(&self, text: &str) -> Option<Token>;
}

/// A token that clients authenticate their requests with.
///
/// It's sent over the wire in the text form given by a [`TokenFormat`].
pub struct Token(
    /// The first byte is the token's [`TokenKind`], and the rest are random.
    [u8; 32],
);

/// What a [`Token`] authenticates.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenKind {
    /// A CI job.
    CiJob = 0,

    /// A machine, like a laptop or a build server.
    Machine = 1,
}

/// The SHA-256 hash of a [`Token`].
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TokenHash([u8; 32]);

impl Token {
    /// Returns the token's bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Creates a token from its bytes.
    ///
    /// Returns `None` if the bytes aren't a token's.
    #[inline]
    pub fn from_bytes(bytes: [u8; 32]) -> Option<Self> {
        TokenKind::from_byte(bytes[0])?;
        Some(Self(bytes))
    }

    /// Returns the token's hash.
    #[inline]
    pub fn hash(&self) -> TokenHash {
        TokenHash(Sha256::digest(self.0).into())
    }

    /// Returns what the token authenticates.
    #[inline]
    pub fn kind(&self) -> TokenKind {
        TokenKind::from_byte(self.0[0]).expect("checked on creation")
    }

    /// Creates a new random token of the given kind.
    pub(crate) fn generate(kind: TokenKind, rng: &mut impl CryptoRng) -> Self {
        let mut bytes = [0; 32];
        rng.fill_bytes(&mut bytes[1..]);
        bytes[0] = kind as u8;
        Self(bytes)
    }
}

impl TokenKind {
    #[inline]
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::CiJob),
            1 => Some(Self::Machine),
            _ => None,
        }
    }
}

impl TokenHash {
    /// Returns the hash's bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token(..)")
    }
}

impl fmt::Debug for TokenHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenHash(")?;
        self.0.iter().try_for_each(|byte| write!(f, "{byte:02x}"))?;
        f.write_str(")")
    }
}
