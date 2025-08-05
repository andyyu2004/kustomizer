use crate::{manifest::Str, resmap::ResourceMap};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

#[async_trait::async_trait]
impl Transformer for NamespaceTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        for resource in resources.iter_mut() {
            resource.metadata_mut().set("namespace", self.0.clone());
        }

        Ok(())
    }
}
