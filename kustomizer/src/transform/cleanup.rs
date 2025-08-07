use crate::resmap::ResourceMap;

use super::Transformer;

#[derive(Default)]
pub struct CleanupTransformer(());

#[async_trait::async_trait]
impl Transformer for CleanupTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        for resource in resources.iter_mut() {
            resource.metadata_mut().clear_internal_fields();
        }

        Ok(())
    }
}
