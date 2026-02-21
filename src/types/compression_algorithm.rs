use core::fmt;
use core::str::FromStr;

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// TODO: docs.
    Bzip2,
    /// TODO: docs.
    Gzip,
    /// TODO: docs.
    None,
    /// TODO: docs.
    Xz,
    /// TODO: docs.
    Zstd,
}

/// The type of error that can occur when parsing a string into a
/// [`CompressionAlgorithm`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum CompressionAlgorithmFromStrError {
    /// The string did not match any of the names of the support compression
    /// algorithms.
    #[display(
        "Invalid compression algorithm, expected one of {}",
        CommaSeparated(CompressionAlgorithm::variants().into_iter().map(CompressionAlgorithm::as_str))
    )]
    InvalidAlgorithm,
}

struct CommaSeparated<T>(T);

impl CompressionAlgorithm {
    const BZIP2: &'static str = "bzip2";
    const GZIP: &'static str = "gzip";
    const NONE: &'static str = "none";
    const XZ: &'static str = "xz";
    const ZSTD: &'static str = "zstd";

    /// Returns the name of this compression algorithm as it appears in the
    /// `compression` field of a Nix store path.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bzip2 => Self::BZIP2,
            Self::Gzip => Self::GZIP,
            Self::None => Self::NONE,
            Self::Xz => Self::XZ,
            Self::Zstd => Self::ZSTD,
        }
    }

    const fn variants() -> [Self; 5] {
        [Self::Bzip2, Self::Gzip, Self::None, Self::Xz, Self::Zstd]
    }
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for CompressionAlgorithm {
    type Err = CompressionAlgorithmFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::BZIP2 => Ok(Self::Bzip2),
            Self::GZIP => Ok(Self::Gzip),
            Self::NONE => Ok(Self::None),
            Self::XZ => Ok(Self::Xz),
            Self::ZSTD => Ok(Self::Zstd),
            _ => Err(CompressionAlgorithmFromStrError::InvalidAlgorithm),
        }
    }
}

impl<T> fmt::Display for CommaSeparated<T>
where
    T: Clone + IntoIterator,
    T::Item: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.clone().into_iter();
        if let Some(first) = iter.next() {
            first.fmt(f)?;
            for item in iter {
                write!(f, ", {item}")?;
            }
        }
        Ok(())
    }
}
