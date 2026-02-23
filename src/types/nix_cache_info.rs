use core::fmt;

use crate::types::NixStoreLiteral;

/// TODO: docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixCacheInfo<StoreDir = NixStoreLiteral> {
    /// TODO: docs.
    pub store_dir: StoreDir,
    /// TODO: docs.
    pub want_mass_query: bool,
    /// TODO: docs.
    pub priority: u32,
}

impl<StoreDir: fmt::Display> fmt::Display for NixCacheInfo<StoreDir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "StoreDir: {}", self.store_dir)?;
        writeln!(f, "WantMassQuery: {}", self.want_mass_query as u8)?;
        writeln!(f, "Priority: {}", self.priority)
    }
}
