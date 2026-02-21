use core::fmt::{self, Write};
use core::num::NonZeroU64;

use smallvec::SmallVec;

use crate::types::{
    CompressionAlgorithm,
    ContentAddress,
    Nix32HashStr,
    NixStoreBaseName,
    NixStorePath,
};

/// TODO: docs.
pub struct NarInfo {
    /// TODO: docs.
    store_path: NixStorePath,
    /// TODO: docs.
    url: String,
    /// SHA-256 hash of the uncompressed NAR file.
    nar_hash: Nix32HashStr<32>,
    /// Byte size of the uncompressed NAR file.
    nar_size: NonZeroU64,
    /// TODO: docs.
    compression: CompressionAlgorithm,
    /// SHA-256 hash of the compressed NAR file.
    file_hash: Nix32HashStr<32>,
    /// Byte size of the compressed NAR file.
    file_size: NonZeroU64,
    /// TODO: docs.
    references: SmallVec<[NixStoreBaseName; 2]>,
    /// TODO: docs.
    deriver: Option<NixStoreBaseName>,
    /// TODO: docs.
    signatures: SmallVec<[String; 1]>,
    /// TODO: docs.
    content_address: Option<ContentAddress>,
}

impl fmt::Display for NarInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "StorePath: {}", self.store_path)?;
        writeln!(f, "URL: {}", self.url)?;
        writeln!(f, "Compression: {}", self.compression)?;
        writeln!(f, "FileHash: {}", self.file_hash)?;
        writeln!(f, "FileSize: {}", self.file_size)?;
        writeln!(f, "NarHash: {}", self.nar_hash)?;
        writeln!(f, "NarSize: {}", self.nar_size)?;

        if !self.references.is_empty() {
            write!(f, "References:")?;
            for reference in &self.references {
                write!(f, " {}", reference)?;
            }
            f.write_char('\n')?;
        }

        write!(f, "Deriver: ")?;

        match &self.deriver {
            Some(basename) => write!(f, "{}", basename)?,
            None => f.write_str("unknown-deriver")?,
        }

        f.write_char('\n')?;

        if !self.signatures.is_empty() {
            write!(f, "Sig:")?;
            for signature in &self.signatures {
                write!(f, " {}", signature)?;
            }
            f.write_char('\n')?;
        }

        if let Some(content_address) = &self.content_address {
            writeln!(f, "CA: {}", content_address)?;
        }

        Ok(())
    }
}
