use core::fmt::{self, Write};
use core::num::NonZeroU64;
use core::str::FromStr;

use smallvec::SmallVec;

use crate::types::{
    CompressionAlgorithm,
    ContentAddress,
    Nix32Hash,
    NixSignature,
    NixStoreBaseName,
    NixStorePath,
};

/// TODO: docs.
pub struct NarInfo<NarUrl> {
    /// TODO: docs.
    pub store_path: NixStorePath,
    /// TODO: docs.
    pub url: NarUrl,
    /// TODO: docs.
    pub compression: CompressionAlgorithm,
    /// SHA-256 hash of the compressed NAR file.
    pub file_hash: Nix32Hash<32>,
    /// Byte size of the compressed NAR file.
    pub file_size: NonZeroU64,
    /// SHA-256 hash of the uncompressed NAR file.
    pub nar_hash: Nix32Hash<32>,
    /// Byte size of the uncompressed NAR file.
    pub nar_size: NonZeroU64,
    /// TODO: docs.
    pub references: SmallVec<[NixStoreBaseName; 2]>,
    /// TODO: docs.
    pub deriver: Option<NixStoreBaseName>,
    /// TODO: docs.
    pub signatures: SmallVec<[NixSignature; 1]>,
    /// TODO: docs.
    pub content_address: Option<ContentAddress>,
}

/// The type of error that can occur when parsing a string into a
/// [`NarInfo`].
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, derive_more::Display, cauchy::Error,
)]
pub enum NarInfoFromStrError<NarUrlFromStrError> {
    /// The URL field did not parse successfully.
    #[display("{_0}")]
    InvalidUrl(NarUrlFromStrError),

    /// The given required NARInfo field was missing from the input string.
    #[display("missing required field: {_0}")]
    MissingRequiredField(&'static str),
}

impl<NarUrl> NarInfo<NarUrl> {
    /// TODO: docs.
    pub fn with_url<NewUrl>(self, new_nar_url: NewUrl) -> NarInfo<NewUrl> {
        NarInfo {
            store_path: self.store_path,
            url: new_nar_url,
            compression: self.compression,
            file_hash: self.file_hash,
            file_size: self.file_size,
            nar_hash: self.nar_hash,
            nar_size: self.nar_size,
            references: self.references,
            deriver: self.deriver,
            signatures: self.signatures,
            content_address: self.content_address,
        }
    }
}

impl<NarUrl: fmt::Display> fmt::Display for NarInfo<NarUrl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "StorePath: {}", self.store_path)?;
        writeln!(f, "URL: {}", self.url)?;
        writeln!(f, "Compression: {}", self.compression)?;
        writeln!(f, "FileHash: sha256:{}", self.file_hash)?;
        writeln!(f, "FileSize: {}", self.file_size)?;
        writeln!(f, "NarHash: sha256:{}", self.nar_hash)?;
        writeln!(f, "NarSize: {}", self.nar_size)?;

        write!(f, "References:")?;
        for reference in &self.references {
            write!(f, " {}", reference)?;
        }
        f.write_char('\n')?;

        if let Some(deriver) = &self.deriver {
            writeln!(f, "Deriver: {deriver}")?;
        }

        for signature in &self.signatures {
            writeln!(f, "Sig: {}", signature)?;
        }

        if let Some(content_address) = &self.content_address {
            writeln!(f, "CA: {}", content_address)?;
        }

        Ok(())
    }
}

impl<NarUrl> FromStr for NarInfo<NarUrl>
where
    NarUrl: FromStr,
{
    type Err = NarInfoFromStrError<NarUrl::Err>;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
