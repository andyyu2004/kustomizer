use indexmap::IndexMap;

use crate::{fieldspec, manifest::Str};

use super::{ResourceMap, Transformer};

// This implementation is not right, see following. It should take some well known paths and only patch those.
// internal/konfig/builtinpluginconsts/commonannotations.go
pub struct AnnotationTransformer<'a>(pub &'a IndexMap<Str, Str>);

impl Transformer for AnnotationTransformer<'_> {
    fn transform(&mut self, resources: &mut ResourceMap) {
        if self.0.is_empty() {
            return;
        }

        let field_specs = &fieldspec::Builtin::get().common_annotations;

        for resource in resources.iter_mut() {
            for field_spec in field_specs.iter() {
                field_spec.apply(resource, |annotations| {
                    for (key, value) in self.0 {
                        annotations.insert(
                            serde_yaml::Value::String(key.to_string()),
                            serde_yaml::Value::String(value.to_string()),
                        );
                    }
                });
            }
        }
    }
}
