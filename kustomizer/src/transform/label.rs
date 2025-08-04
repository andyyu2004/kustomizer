use crate::{fieldspec, manifest::Label, resmap::ResourceMap};

use super::Transformer;

pub struct LabelTransformer<'a>(pub &'a [Label]);

impl Transformer for LabelTransformer<'_> {
    fn transform(&mut self, resources: &mut ResourceMap) {
        if self.0.is_empty() {
            return;
        }

        let builtins = fieldspec::Builtin::get();
        // let field_specs = if label.include_selectors {
        //     builtins.common_labels
        // } else {
        //     builtins.metadata_labels
        // };
        let field_specs = &builtins.metadata_labels;

        for resource in resources.iter_mut() {
            for field_spec in field_specs.iter() {
                field_spec.apply(resource, |labels| {
                    for label in self.0 {
                        for (key, value) in &label.pairs {
                            labels.insert(
                                serde_yaml::Value::String(key.to_string()),
                                serde_yaml::Value::String(value.to_string()),
                            );
                        }
                    }
                });
            }
        }
    }
}
