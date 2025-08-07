use std::sync::OnceLock;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

const SPEC_V2_GZ: &[u8] = include_bytes!("./openapi-v2-kubernetes-1.32-minimized.json.gz");

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    definitions: IndexMap<String, Schema>,
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
