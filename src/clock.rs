use core::time::Duration;

/// TODO: docs.
pub trait Clock {
    /// Returns the elapsed time since the UNIX epoch.
    fn unix_ts(&self) -> Duration;
}
