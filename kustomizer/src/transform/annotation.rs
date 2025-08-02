use std::ops::ControlFlow;

use indexmap::IndexMap;

use crate::{
    manifest::Str,
    visit::{VisitMut, VisitorMut},
};

use super::{ResourceMap, Transformer};

pub struct AnnotationTransformer {
    pub annotation: IndexMap<Str, Str>,
}

impl Transformer for AnnotationTransformer {
    fn transform(&mut self, resources: &mut ResourceMap) {
        for resource in resources.iter_mut() {
            resource
                .metadata
                .annotations
                .extend(self.annotation.clone());

            resource.data.visit_with(self);
        }
    }
}

impl VisitorMut for AnnotationTransformer {
    type Break = ();

    fn visit_mapping(&mut self, mapping: &mut serde_yaml::Mapping) -> ControlFlow<Self::Break> {
        if let Some(serde_yaml::Value::Mapping(annotations)) = mapping.get_mut("annotations") {
            for (key, value) in &self.annotation {
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
