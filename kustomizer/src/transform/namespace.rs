use crate::{manifest::Str, patch::openapi, resmap::ResourceMap};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

#[async_trait::async_trait]
impl Transformer for NamespaceTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let spec = openapi::v2::Spec::load();
        for resource in resources.iter_mut() {
            if spec.is_namespaced(resource.gvk()) {
                resource.metadata_mut().set("namespace", self.0.clone());
            } else {
                panic!("{:?} is not a namespaced resource", resource.gvk());
            }
        }

        Ok(())
    }
}
