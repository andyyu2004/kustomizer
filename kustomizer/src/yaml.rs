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

pub fn from_reader<R, T>(reader: R) -> anyhow::Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    serde_saphyr::from_reader(reader).map_err(Into::into)
}

pub fn to_string<T>(value: &T) -> anyhow::Result<String>
where
    T: serde::Serialize,
{
    serde_saphyr::to_string(value).map_err(Into::into)
}

pub fn to_fmt_writer<W: std::fmt::Write, T: serde::Serialize>(
    output: &mut W,
    value: &T,
) -> anyhow::Result<()> {
    serde_saphyr::to_fmt_writer(output, value).map_err(Into::into)
}
