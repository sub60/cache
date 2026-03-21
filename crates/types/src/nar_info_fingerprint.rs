use core::fmt::{self, Write};
use core::num::NonZeroU64;

use crate::{
    NarInfo,
    Nix32Digest,
    NixStoreBaseName,
    NixStoreLiteral,
    NixStorePath,
};

/// TODO: docs.
pub struct NarInfoFingerprint<'nar_info, StoreDir = NixStoreLiteral> {
    version: NarInfoFingerprintVersion,
    store_path: &'nar_info NixStorePath<StoreDir>,
    nar_hash: &'nar_info Nix32Digest<32>,
    nar_size: NonZeroU64,
    references: &'nar_info [NixStoreBaseName],
}

enum NarInfoFingerprintVersion {
    One,
}

impl<'a, StoreDir, Url> From<&'a NarInfo<Url, StoreDir>>
    for NarInfoFingerprint<'a, StoreDir>
{
    fn from(narinfo: &'a NarInfo<Url, StoreDir>) -> Self {
        Self {
            version: NarInfoFingerprintVersion::One,
            store_path: &narinfo.store_path,
            nar_hash: &narinfo.nar_hash,
            nar_size: narinfo.nar_size,
            references: &narinfo.references,
        }
    }
}

impl<StoreDir: fmt::Display> fmt::Display for NarInfoFingerprint<'_, StoreDir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The NARInfo fingerprint format is a semicolon-delimited string with
        // this structure:
        //
        // <version>;<store-path>;<nar-hash>;<nar-size>;<references>
        //
        // where the <references> are a comma-separated list of full store paths
        // of all the references. Note that the `NarInfo::references` field
        // stores a list of store *basenames*, so to convert them to store paths
        // we append the store directory of the StorePath to each of them.

        write!(
            f,
            "{};{};sha256:{};{};",
            self.version, self.store_path, self.nar_hash, self.nar_size
        )?;

        let store_dir = self.store_path.store_dir();
        let num_references = self.references.len();

        for (ref_idx, ref_basename) in self.references.iter().enumerate() {
            NixStorePath::fmt_display(store_dir, ref_basename, f)?;
            if ref_idx + 1 < num_references {
                f.write_char(',')?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for NarInfoFingerprintVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::One => "1",
        })
    }
}
