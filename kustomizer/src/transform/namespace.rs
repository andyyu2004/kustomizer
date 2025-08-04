use crate::{manifest::Str, resmap::ResourceMap};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

impl Transformer for NamespaceTransformer {
    fn transform(&mut self, resources: &mut ResourceMap) {
        for resource in resources.iter_mut() {
            resource.metadata_mut().set("namespace", self.0.clone());
        }
    }
}
