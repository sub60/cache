use axum::body::Bytes;
use nix_types::{NarInfo, StorePath};

/// TODO: docs.
pub trait NixStore: Sized {
    fn add_file(
        &self,
        file_name: &str,
        file_contents: &[u8],
    ) -> impl Future<Output = StorePath>;

    fn delete(&self, store_path: &StorePath) -> impl Future<Output = ()>;

    /// Returns the [`NarInfo`] and the (possibly compressed) NAR bytes for
    /// the given store path.
    ///
    /// The returned [`NarInfo`] has an empty URL field, matching the format
    /// used when storing NarInfos in object storage.
    fn nar_info_and_bytes(
        &self,
        store_path: &StorePath,
    ) -> impl Future<Output = (NarInfo<String>, Bytes)>;

    /// Returns whether the given store path exists in this store.
    fn contains(&self, store_path: &StorePath) -> impl Future<Output = bool>;
}
