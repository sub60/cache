use axum::body::Bytes;
use cache::types::{NarInfo, NixStorePath};

/// TODO: docs.
pub trait NixStore: Sized {
    fn add_file(
        &self,
        file_name: &str,
        file_contents: &[u8],
    ) -> impl Future<Output = NixStorePath>;

    fn remove_store_path(
        &self,
        store_path: &NixStorePath,
    ) -> impl Future<Output = ()>;

    /// Returns the [`NarInfo`] for the given store path.
    ///
    /// The returned [`NarInfo`] has an empty URL field, matching the format
    /// used when storing NarInfos in object storage.
    fn narinfo(
        &self,
        store_path: &NixStorePath,
    ) -> impl Future<Output = NarInfo<String>>;

    /// Returns the NAR bytes for the given store path.
    fn nar_bytes(
        &self,
        store_path: &NixStorePath,
    ) -> impl Future<Output = Bytes>;

    /// Returns whether the given store path exists in this store.
    fn contains(&self, store_path: &NixStorePath)
    -> impl Future<Output = bool>;
}
