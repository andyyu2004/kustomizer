use serde::{Deserialize, de::DeserializeOwned};

pub fn from_str<'de, T>(s: &'de str) -> anyhow::Result<T>
where
    T: Deserialize<'de>,
{
    serde_yaml::from_str(s).map_err(Into::into)
}

pub fn from_slice<'de, T>(s: &'de [u8]) -> anyhow::Result<T>
where
    T: Deserialize<'de>,
{
    serde_yaml::from_slice(s).map_err(Into::into)
}

pub fn to_string<T>(value: &T) -> anyhow::Result<String>
where
    T: serde::Serialize,
{
    serde_yaml::to_string(value).map_err(Into::into)
}

pub fn from_reader<R, T>(reader: R) -> anyhow::Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    serde_yaml::from_reader(reader).map_err(Into::into)
}
