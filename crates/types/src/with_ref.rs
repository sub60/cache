/// A weaker version of [`AsRef`] that takes a callback instead of returning a
/// reference.
pub trait WithRef<T: ?Sized> {
    /// TODO: docs.
    fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R;
}

impl WithRef<[u8]> for ed25519_dalek::Signature {
    fn with_ref<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        f(&self.to_bytes())
    }
}
