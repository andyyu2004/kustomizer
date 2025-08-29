use serde::{Deserialize, Serialize};

use crate::{
    manifest::{ImageTag, TypeMeta, apiversion, kind},
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

impl From<ImageTag> for ImageTagTransformer {
    fn from(image_tag: ImageTag) -> Self {
        Self {
            type_meta: TypeMeta {
                api_version: Some(apiversion::Builtin),
                kind: Some(kind::ImageTagTransformer),
            },
            metadata: Metadata::default(),
            image_tag,
        }
    }
}

impl Transformer for ImageTagTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let field_specs = &crate::fieldspec::Builtin::load().images;

        for resource in resources.iter_mut() {
            field_specs.apply::<String>(resource, |image_ref| {
                if *image_ref == self.image_tag.name {
                    let new_name = if self.image_tag.new_name.is_empty() {
                        self.image_tag.name.clone()
                    } else {
                        self.image_tag.new_name.clone()
                    };
                    *image_ref = format!("{new_name}:{}", self.image_tag.new_tag);
                }

                Ok(())
            })?;
        }
        Ok(())
    }
}
