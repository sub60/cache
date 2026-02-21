use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

/// Nix's custom base32 alphabet for hashes, which includes all digits and
/// all lowercase letters except 'e', 'o', 't', and 'u'.
const ALPHABET: &[u8; 32] = b"0123456789abcdfghijklmnpqrsvwxyz";

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Nix32Hash<const NUM_BYTES: usize>([u8; NUM_BYTES]);

/// TODO: docs.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Nix32HashStr<const NUM_BYTES: usize>(
    [u8; num_chars_of_bytes(NUM_BYTES)],
)
where
    [(); num_chars_of_bytes(NUM_BYTES)]:;

/// The type of error that can occur when parsing a string into a
/// [`NixStoreHash`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum Nix32HashFromStrError<const NUM_BYTES: usize> {
    /// The string was not exactly 32 characters long.
    #[display(
        "Nix store hash must be exactly {} characters long",
        num_chars_of_bytes(NUM_BYTES)
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

impl<const NUM_BYTES: usize> Nix32Hash<NUM_BYTES> {
    /// Returns the hash's raw bytes.
    pub const fn as_bytes(&self) -> &[u8; NUM_BYTES] {
        &self.0
    }

    /// Creates a new hash from the given raw bytes.
    pub const fn new(bytes: [u8; NUM_BYTES]) -> Self {
        Self(bytes)
    }

    /// TODO: docs.
    pub const fn to_str(&self) -> Nix32HashStr<NUM_BYTES>
    where
        [(); num_chars_of_bytes(NUM_BYTES)]:,
    {
        let mut chars = [0; _];
        encode_bytes_into_chars(self.as_bytes(), &mut chars);
        Nix32HashStr(chars)
    }
}

impl<const NUM_BYTES: usize> Nix32HashStr<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    /// TODO: docs.
    pub const NUM_CHARACTERS: usize = num_chars_of_bytes(NUM_BYTES);

    /// TODO: docs.
    pub const fn as_str(&self) -> &str {
        // SAFETY: todo.
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }

    /// TODO: docs.
    pub const fn to_bytes(&self) -> Nix32Hash<NUM_BYTES>
    where
        [(); num_chars_of_bytes(NUM_BYTES)]:,
    {
        let chars = &self.0;
        let mut bytes = [0; _];
        decode_chars_into_bytes(chars, &mut bytes);
        Nix32Hash(bytes)
    }

    const fn is_byte_valid(byte: u8) -> bool {
        const VALID_BYTES: [bool; 256] = {
            let mut lut = [false; 256];
            let mut idx = 0;

            while idx < ALPHABET.len() {
                lut[ALPHABET[idx] as usize] = true;
                idx += 1;
            }

            lut
        };

        VALID_BYTES[byte as usize]
    }

    fn validate(s: &str) -> Result<(), Nix32HashFromStrError<NUM_BYTES>> {
        if s.len() != NUM_BYTES {
            return Err(Nix32HashFromStrError::InvalidLength);
        }

        for (offset, byte) in s.bytes().enumerate() {
            if !Self::is_byte_valid(byte) {
                let character = s[offset..]
                    .chars()
                    .next()
                    .expect("offset must be less that string length");
                return Err(Nix32HashFromStrError::InvalidCharacter {
                    character,
                    offset,
                });
            }
        }

        Ok(())
    }
}

impl<const NUM_BYTES: usize> fmt::Display for Nix32Hash<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_str().as_ref().fmt(f)
    }
}

impl<const NUM_BYTES: usize> FromStr for Nix32Hash<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    type Err = Nix32HashFromStrError<NUM_BYTES>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Nix32HashStr::<NUM_BYTES>::validate(s)?;
        let chars = s.as_bytes();
        let mut bytes = [0; _];
        decode_chars_into_bytes(chars, &mut bytes);
        Ok(Self(bytes))
    }
}

impl<'de, const NUM_BYTES: usize> serde::de::Deserialize<'de>
    for Nix32Hash<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = Cow::<'de, str>::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl<const NUM_BYTES: usize> fmt::Debug for Nix32HashStr<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<const NUM_BYTES: usize> fmt::Display for Nix32HashStr<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const NUM_BYTES: usize> AsRef<str> for Nix32HashStr<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const NUM_BYTES: usize> FromStr for Nix32HashStr<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    type Err = Nix32HashFromStrError<NUM_BYTES>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        let chars = s.as_bytes().try_into().expect("string has been validated");
        Ok(Self(chars))
    }
}

impl<'de, const NUM_BYTES: usize> serde::de::Deserialize<'de>
    for Nix32HashStr<NUM_BYTES>
where
    [(); num_chars_of_bytes(NUM_BYTES)]:,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = Cow::<'de, str>::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[doc(hidden)]
pub const fn num_chars_of_bytes(num_bytes: usize) -> usize {
    (num_bytes * 8 - 1) / 5 + 1
}

const fn encode_bytes_into_chars(bytes: &[u8], chars: &mut [u8]) {
    let mut char_idx = 0;
    while char_idx < chars.len() {
        let b = (chars.len() - char_idx - 1) * 5;
        let i = b / 8;
        let j = b % 8;

        let mut c = bytes[i] >> j;
        if j != 0 && i + 1 < bytes.len() {
            c |= bytes[i + 1] << (8 - j);
        }

        chars[char_idx] = ALPHABET[(c & 0x1f) as usize];
        char_idx += 1;
    }
}

const fn decode_chars_into_bytes(chars: &[u8], bytes: &mut [u8]) {
    let mut n = 0;
    while n < chars.len() {
        let ch = chars[chars.len() - n - 1];
        let digit = decode_digit(ch);

        let b = n * 5;
        let i = b / 8;
        let j = b % 8;

        bytes[i] |= digit.wrapping_shl(j as u32);

        if j != 0 {
            let carry = digit >> (8 - j);
            if i + 1 < bytes.len() {
                bytes[i + 1] |= carry;
            }
        }
        n += 1;
    }
}

const fn decode_digit(byte: u8) -> u8 {
    let mut idx = 0;

    while idx < ALPHABET.len() {
        if ALPHABET[idx] == byte {
            return idx as u8;
        }
        idx += 1;
    }

    panic!("Nix32HashStr is always validated")
}
