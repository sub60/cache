//! The endpoints of the cache's HTTP API, one per module.

pub mod auth_github_actions;
pub mod nar_get;
pub mod nar_head;
pub mod nar_multiparts;
pub mod nar_multiparts_complete;
pub mod nar_multiparts_parts;
pub mod nar_put;
pub mod narinfo_get;
pub mod narinfo_put;
pub mod nix_cache_info;
