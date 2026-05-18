use core::fmt;
use core::str::FromStr;

/// A type whose [`Display`](fmt::Display) impl is "/nix/store".
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NixStoreLiteral;

/// The type of error that can occur when parsing a string into a
/// [`NixStoreLiteral`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
#[display("Nix store literal must be exactly \"{}\"", NixStoreLiteral::STR)]
pub struct NixStoreLiteralFromStrError;

impl NixStoreLiteral {
    /// The string literal `"/nix/store"`.
    pub const STR: &'static str = "/nix/store";
}

impl fmt::Debug for NixStoreLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl fmt::Display for NixStoreLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(windows)]
        {
            compile_error!(
                "NixStoreLiteral does not have a valid Display impl on Windows"
            );
        }
        #[cfg(not(windows))]
        {
            fmt::Display::fmt(Self::STR, f)
        }
    }
}

impl FromStr for NixStoreLiteral {
    type Err = NixStoreLiteralFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == Self::STR { Ok(Self) } else { Err(NixStoreLiteralFromStrError) }
    }
}
