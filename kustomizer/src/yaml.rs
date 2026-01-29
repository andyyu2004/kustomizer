use std::io::Read;

use serde::de::DeserializeOwned;

pub fn from_str<T>(s: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    serde_saphyr::from_str(s).map_err(Into::into)
}

pub fn from_slice<T>(s: &[u8]) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    serde_saphyr::from_slice(s).map_err(Into::into)
}

pub fn from_reader<T>(reader: impl Read) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    serde_saphyr::from_reader(reader).map_err(Into::into)
}

const OPTS: serde_saphyr::SerializerOptions = serde_saphyr::SerializerOptions {
    prefer_block_scalars: true,
    empty_as_braces: true,
    indent_step: 2,
    anchor_generator: None,
    min_fold_chars: 32,
    folded_wrap_chars: 80,
    tagged_enums: false,
};

pub fn to_string<T>(value: &T) -> anyhow::Result<String>
where
    T: serde::Serialize,
{
    serde_saphyr::to_string_with_options(value, OPTS).map_err(Into::into)
}

pub fn to_fmt_writer<W: std::fmt::Write, T: serde::Serialize>(
    output: &mut W,
    value: &T,
) -> anyhow::Result<()> {
    serde_saphyr::to_fmt_writer_with_options(output, value, OPTS).map_err(Into::into)
}
