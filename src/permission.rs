/// What a request must be allowed to do to a resource, like a cache.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Permission {
    /// Reading the resource.
    Read,

    /// Writing to the resource.
    Write,
}
