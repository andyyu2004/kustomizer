use std::ops::ControlFlow;

use indexmap::IndexMap;

use crate::{
    manifest::Str,
    visit::{VisitMut, VisitorMut},
};

use super::{ResourceMap, Transformer};

pub struct AnnotationTransformer<'a>(pub &'a IndexMap<Str, Str>);

impl Transformer for AnnotationTransformer<'_> {
    fn transform(&mut self, resources: &mut ResourceMap) {
        if self.0.is_empty() {
            return;
        }

        for resource in resources.iter_mut() {
            resource
                .metadata
                .annotations
                .extend(self.0.iter().map(|(k, v)| (k.clone(), v.clone())));

            resource.data.visit_with(self);
        }
    }
}

impl VisitorMut for AnnotationTransformer<'_> {
    type Break = ();

    fn visit_mapping(&mut self, mapping: &mut serde_yaml::Mapping) -> ControlFlow<Self::Break> {
        // Matching on `metadata` key is a bit of a hack.
        if let Some(serde_yaml::Value::Mapping(metadata)) = mapping.get_mut("metadata") {
            let annotations = metadata
                .entry(serde_yaml::Value::String("annotations".to_string()))
                .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()))
                .as_mapping_mut()
                .unwrap();
            for (key, value) in self.0 {
                annotations.insert(
                    serde_yaml::Value::String(key.to_string()),
                    serde_yaml::Value::String(value.to_string()),
                );
            }

            return ControlFlow::Break(());
        }

        self.walk_mapping(mapping)
    }
}
