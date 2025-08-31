use crate::resmap::ResourceMap;

use super::Transformer;

#[derive(Default)]
pub struct CleanupTransformer(());

impl Transformer for CleanupTransformer {
    #[tracing::instrument(skip_all)]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        for resource in resources.iter_mut() {
            resource.metadata_mut().clear_internal_fields();
        }

        Ok(())
    }
}
