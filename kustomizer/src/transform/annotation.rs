use indexmap::IndexMap;

use crate::{fieldspec, manifest::Str};

use super::{ResourceMap, Transformer};

pub struct AnnotationTransformer<'a>(pub &'a IndexMap<Str, Str>);

#[async_trait::async_trait]
impl Transformer for AnnotationTransformer<'_> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }

        let field_specs = &fieldspec::Builtin::get().common_annotations;

        for resource in resources.iter_mut() {
            field_specs.apply(resource, |annotations| {
                let annotations = annotations
                    .as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("expected a yaml mapping for annotations"))?;

                for (key, value) in self.0 {
                    annotations.insert(
                        key.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
                Ok(())
            })?;
        }

        Ok(())
    }
}
