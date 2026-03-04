use cache::types::NixStorePath;

/// TODO: docs.
pub trait NixStore: Sized {
    fn add_file(
        &self,
        file_name: &str,
        file_contents: &[u8],
    ) -> impl Future<Output = NixStorePath>;
}
