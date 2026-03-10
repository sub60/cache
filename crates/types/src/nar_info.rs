use core::fmt::{self, Write};
use core::num::NonZeroU64;
use core::str::FromStr;

use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{
    CompressionAlgorithm,
    ContentAddress,
    Nix32Digest,
    NixHash,
    NixSignature,
    NixStoreBaseName,
    NixStorePath,
};

const STORE_PATH_FIELD_NAME: &str = "StorePath";
const URL_FIELD_NAME: &str = "URL";
const COMPRESSION_FIELD_NAME: &str = "Compression";
const FILE_HASH_FIELD_NAME: &str = "FileHash";
const FILE_SIZE_FIELD_NAME: &str = "FileSize";
const NAR_HASH_FIELD_NAME: &str = "NarHash";
const NAR_SIZE_FIELD_NAME: &str = "NarSize";
const REFERENCES_FIELD_NAME: &str = "References";
const DERIVER_FIELD_NAME: &str = "Deriver";
const SIGNATURE_FIELD_NAME: &str = "Sig";
const CONTENT_ADDRESS_FIELD_NAME: &str = "CA";

/// TODO: docs.
#[derive(Debug, PartialEq, Eq)]
pub struct NarInfo<NarUrl> {
    /// TODO: docs.
    pub store_path: NixStorePath,
    /// TODO: docs.
    pub url: NarUrl,
    /// TODO: docs.
    pub compression: CompressionAlgorithm,
    /// SHA-256 hash of the compressed NAR file.
    pub file_hash: Nix32Digest<32>,
    /// Byte size of the compressed NAR file.
    pub file_size: NonZeroU64,
    /// SHA-256 hash of the uncompressed NAR file.
    pub nar_hash: Nix32Digest<32>,
    /// Byte size of the uncompressed NAR file.
    pub nar_size: NonZeroU64,
    /// TODO: docs.
    pub references: SmallVec<[NixStoreBaseName; 2]>,
    /// TODO: docs.
    pub deriver: Option<NixStoreBaseName>,
    /// TODO: docs.
    pub signatures: SmallVec<[NixSignature; 1]>,
    /// TODO: docs.
    pub content_address: Option<ContentAddress>,
}

/// The type of error that can occur when parsing a string into a
/// [`NarInfo`].
#[derive(
    Debug, cauchy::PartialEq, cauchy::Eq, derive_more::Display, cauchy::Error,
)]
pub enum NarInfoFromStrError<NarUrlFromStrError> {
    /// The given field was declared more than once in the input string.
    #[display("duplicate field: {_0}")]
    DuplicateField(SmolStr),

    /// The input string contained an invalid field.
    #[display("invalid field name: {_0:?}")]
    InvalidFieldName(SmolStr),

    /// The input string contained a line that did not contain the expected ": "
    /// separator between the field name and its value.
    #[display("missing ': ' separator in line {_0:?}")]
    MissingSeparator(SmolStr),

    /// The given required NARInfo field was missing from the input string.
    #[display("missing required field: {_0:?}")]
    MissingRequiredField(&'static str),

    /// The `FileHash` field used a hash algorithm that was not SHA-256.
    #[display("expected '{FILE_HASH_FIELD_NAME}' to be a SHA-256 hash")]
    FileHashNotSha256,

    /// The `NarHash` field used a hash algorithm that was not SHA-256.
    #[display("expected '{NAR_HASH_FIELD_NAME}' to be a SHA-256 hash")]
    NarHashNotSha256,

    /// The `StorePath` field did not parse successfully.
    #[display("invalid '{STORE_PATH_FIELD_NAME}': {_0}")]
    StorePath(<NixStorePath as FromStr>::Err),

    /// The `URL` field did not parse successfully.
    #[display("invalid '{URL_FIELD_NAME}': {_0}")]
    Url(NarUrlFromStrError),

    /// The `Compression` field did not parse successfully.
    #[display("invalid '{COMPRESSION_FIELD_NAME}': {_0}")]
    Compression(<CompressionAlgorithm as FromStr>::Err),

    /// The `FileHash` field did not parse successfully.
    #[display("invalid '{FILE_HASH_FIELD_NAME}': {_0}")]
    FileHash(<NixHash as FromStr>::Err),

    /// The `FileSize` field did not parse successfully.
    #[display("invalid '{FILE_SIZE_FIELD_NAME}': {_0}")]
    FileSize(<NonZeroU64 as FromStr>::Err),

    /// The `NarHash` field did not parse successfully.
    #[display("invalid '{NAR_HASH_FIELD_NAME}': {_0}")]
    NarHash(<NixHash as FromStr>::Err),

    /// The `NarSize` field did not parse successfully.
    #[display("invalid '{NAR_SIZE_FIELD_NAME}': {_0}")]
    NarSize(<NonZeroU64 as FromStr>::Err),

    /// The `References` field did not parse successfully.
    #[display("invalid '{REFERENCES_FIELD_NAME}': {_0}")]
    Reference(<NixStoreBaseName as FromStr>::Err),

    /// The `Deriver` field did not parse successfully.
    #[display("invalid '{DERIVER_FIELD_NAME}': {_0}")]
    Deriver(<NixStoreBaseName as FromStr>::Err),

    /// The `Sig` field did not parse successfully.
    #[display("invalid '{SIGNATURE_FIELD_NAME}': {_0}")]
    Signature(#[partial_eq(skip)] <NixSignature as FromStr>::Err),

    /// The `CA` field did not parse successfully.
    #[display("invalid '{CONTENT_ADDRESS_FIELD_NAME}': {_0}")]
    ContentAddress(<ContentAddress as FromStr>::Err),
}

impl<NarUrl> NarInfo<NarUrl> {
    /// TODO: docs.
    pub fn with_url<NewUrl>(self, new_nar_url: NewUrl) -> NarInfo<NewUrl> {
        NarInfo {
            store_path: self.store_path,
            url: new_nar_url,
            compression: self.compression,
            file_hash: self.file_hash,
            file_size: self.file_size,
            nar_hash: self.nar_hash,
            nar_size: self.nar_size,
            references: self.references,
            deriver: self.deriver,
            signatures: self.signatures,
            content_address: self.content_address,
        }
    }
}

impl<NarUrl: fmt::Display> fmt::Display for NarInfo<NarUrl> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{STORE_PATH_FIELD_NAME}: {}", self.store_path)?;
        writeln!(f, "{URL_FIELD_NAME}: {}", self.url)?;
        writeln!(f, "{COMPRESSION_FIELD_NAME}: {}", self.compression)?;
        writeln!(f, "{FILE_HASH_FIELD_NAME}: sha256:{}", self.file_hash)?;
        writeln!(f, "{FILE_SIZE_FIELD_NAME}: {}", self.file_size)?;
        writeln!(f, "{NAR_HASH_FIELD_NAME}: sha256:{}", self.nar_hash)?;
        writeln!(f, "{NAR_SIZE_FIELD_NAME}: {}", self.nar_size)?;

        write!(f, "{REFERENCES_FIELD_NAME}:")?;
        if self.references.is_empty() {
            f.write_char(' ')?;
        } else {
            for reference in &self.references {
                write!(f, " {}", reference)?;
            }
        }
        f.write_char('\n')?;

        if let Some(deriver) = &self.deriver {
            writeln!(f, "{DERIVER_FIELD_NAME}: {deriver}")?;
        }

        for signature in &self.signatures {
            writeln!(f, "{SIGNATURE_FIELD_NAME}: {}", signature)?;
        }

        if let Some(content_address) = &self.content_address {
            writeln!(f, "{CONTENT_ADDRESS_FIELD_NAME}: {}", content_address)?;
        }

        Ok(())
    }
}

impl<NarUrl> FromStr for NarInfo<NarUrl>
where
    NarUrl: FromStr,
{
    type Err = NarInfoFromStrError<NarUrl::Err>;

    #[expect(clippy::too_many_lines)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Required fields.
        let mut store_path = None;
        let mut url = None;
        let mut compression = None;
        let mut file_hash = None;
        let mut file_size = None;
        let mut nar_hash = None;
        let mut nar_size = None;
        let mut references = None;

        // Optional fields.
        let mut deriver = None;
        let mut signatures = SmallVec::new();
        let mut content_address = None;

        for line in s.lines() {
            let (field_name, field_value) =
                line.split_once(": ").ok_or_else(|| {
                    NarInfoFromStrError::MissingSeparator(line.into())
                })?;

            let is_duplicate_field = match field_name {
                STORE_PATH_FIELD_NAME => {
                    let parsed_store_path = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::StorePath)?;
                    store_path.replace(parsed_store_path).is_some()
                },
                URL_FIELD_NAME => {
                    let parsed_url = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::Url)?;
                    url.replace(parsed_url).is_some()
                },
                COMPRESSION_FIELD_NAME => {
                    let parsed_compression = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::Compression)?;
                    compression.replace(parsed_compression).is_some()
                },
                FILE_HASH_FIELD_NAME => {
                    let NixHash::Sha256(parsed_file_hash) = field_value
                        .parse::<NixHash>()
                        .map_err(NarInfoFromStrError::FileHash)?
                    else {
                        return Err(NarInfoFromStrError::FileHashNotSha256);
                    };
                    file_hash.replace(parsed_file_hash).is_some()
                },
                FILE_SIZE_FIELD_NAME => {
                    let parsed_file_size = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::FileSize)?;
                    file_size.replace(parsed_file_size).is_some()
                },
                NAR_HASH_FIELD_NAME => {
                    let NixHash::Sha256(parsed_nar_hash) = field_value
                        .parse::<NixHash>()
                        .map_err(NarInfoFromStrError::NarHash)?
                    else {
                        return Err(NarInfoFromStrError::NarHashNotSha256);
                    };
                    nar_hash.replace(parsed_nar_hash).is_some()
                },
                NAR_SIZE_FIELD_NAME => {
                    let parsed_nar_size = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::NarSize)?;
                    nar_size.replace(parsed_nar_size).is_some()
                },
                REFERENCES_FIELD_NAME => {
                    let parsed_references = if field_value.is_empty() {
                        SmallVec::new()
                    } else {
                        field_value
                            .split(' ')
                            .map(str::parse)
                            .collect::<Result<SmallVec<[NixStoreBaseName; 2]>, _>>()
                            .map_err(NarInfoFromStrError::Reference)?
                    };
                    references.replace(parsed_references).is_some()
                },
                DERIVER_FIELD_NAME => {
                    let parsed_deriver = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::Deriver)?;
                    deriver.replace(parsed_deriver).is_some()
                },
                SIGNATURE_FIELD_NAME => {
                    let signature = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::Signature)?;
                    signatures.push(signature);
                    false
                },
                CONTENT_ADDRESS_FIELD_NAME => {
                    let parsed_content_address = field_value
                        .parse()
                        .map_err(NarInfoFromStrError::ContentAddress)?;
                    content_address.replace(parsed_content_address).is_some()
                },
                other => {
                    return Err(NarInfoFromStrError::InvalidFieldName(
                        other.into(),
                    ));
                },
            };

            if is_duplicate_field {
                return Err(NarInfoFromStrError::DuplicateField(
                    field_name.into(),
                ));
            }
        }

        let try_block = || {
            Ok(Self {
                store_path: store_path.ok_or(STORE_PATH_FIELD_NAME)?,
                url: url.ok_or(URL_FIELD_NAME)?,
                compression: compression.ok_or(COMPRESSION_FIELD_NAME)?,
                file_hash: file_hash.ok_or(FILE_HASH_FIELD_NAME)?,
                file_size: file_size.ok_or(FILE_SIZE_FIELD_NAME)?,
                nar_hash: nar_hash.ok_or(NAR_HASH_FIELD_NAME)?,
                nar_size: nar_size.ok_or(NAR_SIZE_FIELD_NAME)?,
                references: references.ok_or(REFERENCES_FIELD_NAME)?,
                deriver,
                signatures,
                content_address,
            })
        };

        try_block().map_err(NarInfoFromStrError::MissingRequiredField)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nar_info_serialize_roundtrip_1() {
        let nar_info = NarInfo {
            store_path: "/nix/store/00000000000000000000000000000000-foo"
                .parse()
                .unwrap(),
            url: "some_url".to_owned(),
            compression: CompressionAlgorithm::None,
            file_hash: Nix32Digest::new(&[0u8; _]),
            file_size: NonZeroU64::new(123).unwrap(),
            nar_hash: Nix32Digest::new(&[0u8; _]),
            nar_size: NonZeroU64::new(456).unwrap(),
            references: Default::default(),
            deriver: None,
            signatures: Default::default(),
            content_address: Default::default(),
        };

        let serialized = nar_info.to_string();

        let deserialized = match serialized.parse::<NarInfo<String>>() {
            Ok(nar_info) => nar_info,
            Err(err) => panic!("failed to deserialize NarInfo: {err}"),
        };

        assert_eq!(nar_info, deserialized);
    }
}
