use core::fmt;

use crate::types::NixStorePath;

/// TODO: docs.
#[expect(dead_code)]
pub struct NarInfo {
    store_path: NixStorePath,
    url: String,
}

impl fmt::Display for NarInfo {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
