use crate::{fieldspec, manifest::Label, resmap::ResourceMap};

use super::Transformer;

pub struct LabelTransformer<'a>(pub &'a [Label]);

#[async_trait::async_trait]
impl Transformer for LabelTransformer<'_> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        if self.0.is_empty() {
            return Ok(());
        }

        let builtins = fieldspec::Builtin::load();

        for label in self.0 {
            if label.pairs.is_empty() {
                continue;
            }

            let field_specs = match (label.include_selectors, label.include_templates) {
                (true, _) => &builtins.common_labels,
                (false, true) => &builtins.template_labels,
                (false, false) => &builtins.metadata_labels,
            };

            for resource in resources.iter_mut() {
                field_specs.apply(resource, |l| {
                    let l = l
                        .as_object_mut()
                        .ok_or_else(|| anyhow::anyhow!("expected a yaml mapping for labels"))?;

                    for (key, value) in &label.pairs {
                        l.insert(
                            key.to_string(),
                            serde_json::Value::String(value.to_string()),
                        );
                    }

                    Ok(())
                })?;
            }
        }

        Ok(())
    }
}
