use serde::{Deserialize, Serialize};

use crate::{
    manifest::{Str, TypeMeta, apiversion, kind},
    resource::Metadata,
};

use super::{ResourceMap, Transformer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageTagTransformer {
    #[serde(flatten)]
    type_meta: TypeMeta<apiversion::Builtin, kind::ImageTagTransformer>,
    metadata: Metadata,
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
        let field_specs = &crate::fieldspec::Builtin::get().images;

        for resource in resources.iter_mut() {
            for field_spec in field_specs.iter() {
                field_spec.apply(resource, |image_value| {
                    let image = image_value.as_str().ok_or_else(|| {
                        anyhow::anyhow!("expected a string for image field: {}", field_spec.path)
                    })?;

                    if image == self.image_tag.name {
                        let new_image =
                            format!("{}:{}", self.image_tag.new_name, self.image_tag.new_tag);
                        *image_value = serde_yaml::Value::String(new_image);
                    }

                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}
