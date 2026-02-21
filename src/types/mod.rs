//! TODO: docs.

mod cache_name;
mod content_address;
mod nar_info;
mod nix32_hash;
mod nix_store_base_name;
mod nix_store_literal;
mod nix_store_name;
mod nix_store_path;
mod user_name;

/// The type representing the 32-character hash of a Nix store path.
pub type NixStoreHash = Nix32HashStr<20>;

/// The type of error that can occur when parsing a string into a
/// [`NixStoreHash`].
pub type NixStoreHashFromStrError = Nix32HashFromStrError<20>;

pub use cache_name::{CacheName, CacheNameFromStrError};
pub use content_address::{
    ContentAddress,
    ContentAddressFromStrError,
    ContentAddressHash,
    ContentAddressMethod,
};
pub use nar_info::NarInfo;
pub use nix_store_base_name::{NixStoreBaseName, NixStoreBaseNameFromStrError};
pub use nix_store_literal::{NixStoreLiteral, NixStoreLiteralFromStrError};
pub use nix_store_name::{NixStoreName, NixStoreNameFromStrError};
pub use nix_store_path::{NixStorePath, NixStorePathFromStrError};
pub use nix32_hash::{Nix32Hash, Nix32HashFromStrError, Nix32HashStr};
pub use user_name::{UserName, UserNameFromStrError};
