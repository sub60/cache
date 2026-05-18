//! Sub60's Nix binary cache types.

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![no_std]

extern crate alloc;

mod compression_algorithm;
mod content_address;
mod key_or_signature;
mod nar_file_name;
mod nar_info;
mod nar_info_file_name;
mod nar_info_fingerprint;
mod nix32_digest;
mod nix_cache_info;
mod nix_hash;
mod nix_store_literal;
mod store_basename;
mod store_name;
mod store_path;
mod with_ref;

/// The type representing the 32-character hash of a Nix store path.
pub type StoreHash = Nix32Digest<20>;

/// The type of error that can occur when parsing a string into a [`StoreHash`].
pub type StoreHashFromStrError = Nix32DigestFromStrError<20>;

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
pub use key_or_signature::{
    CachePublicKey,
    CacheSigningKey,
    KeyOrSignature,
    KeyOrSignatureName,
    Signature,
};
pub use nar_file_name::{NarFileName, NarFileNameFromStrError};
pub use nar_info::{NarInfo, NarInfoFromStrError};
pub use nar_info_file_name::{NarInfoFileName, NarInfoFileNameFromStrError};
pub use nar_info_fingerprint::NarInfoFingerprint;
pub use nix_cache_info::NixCacheInfo;
pub use nix_hash::{NixHash, NixHashFromStrError};
pub use nix_store_literal::{NixStoreLiteral, NixStoreLiteralFromStrError};
pub use nix32_digest::{Nix32Digest, Nix32DigestFromStrError};
pub use store_basename::{StoreBaseNameFromStrError, StoreBasename};
pub use store_name::{StoreName, StoreNameFromStrError};
pub use store_path::{StorePath, StorePathFromStrError};
pub use with_ref::WithRef;
