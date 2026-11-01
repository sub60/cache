use core::fmt;

use sub60_cache::{Token, TokenFormat};

/// A [`TokenFormat`] that formats tokens as `test_` followed by their bytes in
/// hex.
#[derive(Copy, Clone, Default)]
pub struct TestTokenFormat;

impl TestTokenFormat {
    const PREFIX: &str = "test_";
}

impl TokenFormat for TestTokenFormat {
    fn display(&self, token: &Token) -> impl fmt::Display {
        struct Display<'a>(&'a Token);

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(TestTokenFormat::PREFIX)?;
                self.0
                    .as_bytes()
                    .iter()
                    .try_for_each(|byte| write!(f, "{byte:02x}"))
            }
        }

        Display(token)
    }

    fn parse(&self, text: &str) -> Option<Token> {
        let hex = text.strip_prefix(Self::PREFIX)?;

        if hex.len() != 64 || !hex.is_ascii() {
            return None;
        }

        let mut bytes = [0; 32];

        for (byte, chunk) in bytes.iter_mut().zip(hex.as_bytes().chunks(2)) {
            let chunk = str::from_utf8(chunk).ok()?;
            *byte = u8::from_str_radix(chunk, 16).ok()?;
        }

        Token::from_bytes(bytes)
    }
}
