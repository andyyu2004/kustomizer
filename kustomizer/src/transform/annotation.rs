use indexmap::IndexMap;

use crate::manifest::Str;

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
            // TODO need to find all annotations fields
        }
    }
}
