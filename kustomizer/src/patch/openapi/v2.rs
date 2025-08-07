use std::{fmt, str::FromStr, sync::OnceLock};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::resource::Gvk;

const SPEC_V2_GZ: &[u8] = include_bytes!("./openapi-v2-kubernetes-1.32-minimized.json.gz");

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    definitions: IndexMap<DefinitionId, Schema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefinitionId {
    gvk: Gvk,
}

impl Serialize for DefinitionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DefinitionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)
            .map_err(|err| serde::de::Error::custom(format!("parsing DefinitionId: {err}")))?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for DefinitionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, kind) = s
            .rsplit_once('.')
            .ok_or_else(|| anyhow::anyhow!("missing kind"))?;

        let (group, version) = s
            .rsplit_once('.')
            .ok_or_else(|| anyhow::anyhow!("missing version"))?;

        Ok(Self {
            gvk: Gvk {
                group: group.into(),
                version: version.into(),
                kind: kind.into(),
            },
        })
    }
}

impl fmt::Display for DefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.gvk)
    }
}

impl Spec {
    pub fn load() -> &'static Self {
        static CACHE: OnceLock<Spec> = OnceLock::new();
        CACHE.get_or_init(|| {
            serde_json::from_reader(flate2::read::GzDecoder::new(SPEC_V2_GZ))
                .expect("test should guarantee this is valid")
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    #[serde(flatten)]
    pub ty: Option<Type>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlineOrRef<T> {
    Ref(Ref),
    Inline(T),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ref {
    #[serde(rename = "$ref")]
    reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Type {
    Object {
        properties: IndexMap<String, Schema>,
    },
    Array {
        items: InlineOrRef<Box<Schema>>,
    },
    String,
    Integer,
    Boolean,
}

#[cfg(test)]
mod tests {
    use super::Spec;

    #[test]
    #[ignore]
    fn minimize_openapi_v2_spec() -> anyhow::Result<()> {
        let reader = flate2::read::GzDecoder::new(
            include_bytes!("./openapi-v2-kubernetes-1.32.json.gz").as_ref(),
        );

        let spec: Spec = serde_json::from_reader(reader)?;

        let file = std::fs::File::create(
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json.gz",
        )?;
        serde_json::to_writer(
            flate2::write::GzEncoder::new(file, flate2::Compression::default()),
            &spec,
        )?;

        // Ensure the spec can be loaded
        super::Spec::load();

        Ok(())
    }
}
