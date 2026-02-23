use core::fmt;
use core::str::FromStr;
use std::borrow::Cow;

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

/// The file extension produced by a given [`CompressionAlgorithm`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompressionExtension {
    /// TODO: docs.
    Bzip2,
    /// TODO: docs.
    Gzip,
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

/// The type of error that can occur when parsing a string into a
/// [`CompressionExtension`].
#[derive(Debug, PartialEq, Eq, derive_more::Display, cauchy::Error)]
pub enum CompressionExtensionFromStrError {
    /// The string did not match any of the names of the support compression
    /// extensions.
    #[display(
        "Invalid compression extension, expected one of {}",
        CommaSeparated(CompressionExtension::variants().into_iter().map(CompressionExtension::as_str))
    )]
    InvalidExtension,
}

struct CommaSeparated<T>(T);

impl CompressionAlgorithm {
    const BZIP2: &'static str = "bzip2";
    const GZIP: &'static str = "gzip";
    const NONE: &'static str = "none";
    const XZ: &'static str = "xz";
    const ZSTD: &'static str = "zstd";

    const NUM_VARIANTS: usize = 5;

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

    /// Returns the name of this compression algorithm as it appears in the
    /// `compression` field of a Nix store path.
    pub fn extension(self) -> Option<CompressionExtension> {
        match self {
            Self::Bzip2 => Some(CompressionExtension::Bzip2),
            Self::Gzip => Some(CompressionExtension::Gzip),
            Self::None => None,
            Self::Xz => Some(CompressionExtension::Xz),
            Self::Zstd => Some(CompressionExtension::Zstd),
        }
    }

    const fn variants() -> [Self; Self::NUM_VARIANTS] {
        [Self::Bzip2, Self::Gzip, Self::None, Self::Xz, Self::Zstd]
    }
}

impl CompressionExtension {
    const BZIP2: &'static str = "bz2";
    const GZIP: &'static str = "gz";
    const XZ: &'static str = "xz";
    const ZSTD: &'static str = "zst";

    /// Returns the compression algorithm that produces files with this
    /// extension.
    pub fn algorithm(self) -> CompressionAlgorithm {
        match self {
            Self::Bzip2 => CompressionAlgorithm::Bzip2,
            Self::Gzip => CompressionAlgorithm::Gzip,
            Self::Xz => CompressionAlgorithm::Xz,
            Self::Zstd => CompressionAlgorithm::Zstd,
        }
    }

    /// Returns the file extension as a string slice.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bzip2 => Self::BZIP2,
            Self::Gzip => Self::GZIP,
            Self::Xz => Self::XZ,
            Self::Zstd => Self::ZSTD,
        }
    }

    const fn variants() -> [Self; CompressionAlgorithm::NUM_VARIANTS - 1] {
        [Self::Bzip2, Self::Gzip, Self::Xz, Self::Zstd]
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

impl fmt::Display for CompressionExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl FromStr for CompressionExtension {
    type Err = CompressionExtensionFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::BZIP2 => Ok(Self::Bzip2),
            Self::GZIP => Ok(Self::Gzip),
            Self::XZ => Ok(Self::Xz),
            Self::ZSTD => Ok(Self::Zstd),
            _ => Err(CompressionExtensionFromStrError::InvalidExtension),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for CompressionExtension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        <Cow<'de, str>>::deserialize(deserializer)?
            .parse::<Self>()
            .map_err(serde::de::Error::custom)
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
