/// A trait whose associated types abstract over all the aspects of the cache
/// server for which we want to have mockable implementations in tests.
pub trait Context: Clone + Send + Sync + 'static {}
