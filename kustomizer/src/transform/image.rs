use serde::{Deserialize, Serialize};

use crate::manifest::Str;

use super::{ResourceMap, Transformer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageTagTransformer {
    image_tag: ImageTag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageTag {
    pub name: Str,
    pub new_name: Str,
    pub new_tag: Str,
}

#[async_trait::async_trait]
impl Transformer for ImageTagTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        Ok(())
    }
}
