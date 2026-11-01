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

/// Deserializes a string with `T`'s [`FromStr`](core::str::FromStr) impl, for
/// use with `#[serde(deserialize_with)]`.
pub(crate) fn deserialize_from_str<'de, D, T>(
    deserializer: D,
) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: core::str::FromStr<Err: core::fmt::Display>,
{
    use serde::Deserialize;

    alloc::borrow::Cow::<'de, str>::deserialize(deserializer)?
        .parse()
        .map_err(serde::de::Error::custom)
}
