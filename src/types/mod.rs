//! TODO: docs.

mod cache_name;
mod compression_algorithm;
mod content_address;
mod nar_file_name;
mod nar_info;
mod nix32_digest;
mod nix_cache_info;
mod nix_hash;
mod nix_key_or_signature;
mod nix_store_basename;
mod nix_store_literal;
mod nix_store_name;
mod nix_store_path;
mod user_name;
mod with_ref;

/// TODO: docs.
pub type NixSignature = NixKeyOrSignature<ed25519_dalek::Signature>;

/// The type representing the 32-character hash of a Nix store path.
pub type NixStoreHash = Nix32Digest<20>;

/// The type of error that can occur when parsing a string into a
/// [`NixStoreHash`].
pub type NixStoreHashFromStrError = Nix32DigestFromStrError<20>;

pub use cache_name::{CacheName, CacheNameFromStrError};
pub use compression_algorithm::{
    CompressionAlgorithm,
    CompressionAlgorithmFromStrError,
    CompressionExtension,
    CompressionExtensionFromStrError,
};
pub use content_address::{
    ContentAddress,
    ContentAddressFromStrError,
    ContentAddressMethod,
    ContentAddressMethodFromStrError,
};
pub use nar_file_name::{NarFileName, NarFileNameFromStrError};
pub use nar_info::{NarInfo, NarInfoFromStrError};
pub use nix_cache_info::NixCacheInfo;
pub use nix_hash::{NixHash, NixHashFromStrError};
pub use nix_key_or_signature::{NixKeyOrSignature, NixKeyOrSignatureName};
pub use nix_store_basename::{NixStoreBaseName, NixStoreBaseNameFromStrError};
pub use nix_store_literal::{NixStoreLiteral, NixStoreLiteralFromStrError};
pub use nix_store_name::{NixStoreName, NixStoreNameFromStrError};
pub use nix_store_path::{NixStorePath, NixStorePathFromStrError};
pub use nix32_digest::{Nix32Digest, Nix32DigestFromStrError};
pub use user_name::{UserName, UserNameFromStrError};
pub use with_ref::WithRef;
