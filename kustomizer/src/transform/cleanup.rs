use crate::resmap::ResourceMap;

use super::Transformer;

#[derive(Default)]
pub struct CleanupTransformer(());

impl Transformer for CleanupTransformer {
    #[tracing::instrument(skip_all, name = "cleanup_transform")]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        for resource in resources.iter_mut() {
            if let Some(mut metadata) = resource.metadata_mut() {
                metadata.clear_internal_fields()
            }
        }

        Ok(())
    }
}
