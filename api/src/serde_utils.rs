//! Helpers for use with serde's field attributes.

/// (De)serializing [`Duration`](core::time::Duration)s as whole seconds, for
/// use with `#[serde(with)]`.
///
/// Serializing drops the sub-second part.
pub(crate) mod secs {
    use core::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(
        duration: &Duration,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(duration.as_secs())
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Duration, D::Error> {
        u64::deserialize(deserializer).map(Duration::from_secs)
    }
}
