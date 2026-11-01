//! Helpers for use with serde's field attributes.

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
