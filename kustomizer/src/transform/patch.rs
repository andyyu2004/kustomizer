use crate::{manifest::Patch, resmap::ResourceMap};

use super::Transformer;

pub struct PatchTransformer<'a> {
    patches: &'a [Patch],
}

impl<'a> PatchTransformer<'a> {
    pub fn new(patches: &'a [Patch]) -> Self {
        Self { patches }
    }
}

#[async_trait::async_trait]
impl Transformer for PatchTransformer<'_> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        for resource in resources.iter_mut() {
            for patch in self.patches {
                match patch {
                    Patch::Json { patch, target } => {
                        if !target.matches(resource) {
                            continue;
                        }

                        todo!("apply JSON patch: {patch:?} to resource: {}", resource.id());
                    }
                    Patch::StrategicMerge { path, target } => {
                        if let Some(target) = target
                            && !target.matches(resource)
                        {
                            continue;
                        }

                        todo!(
                            "apply strategic merge patch: {path:?} to resource: {}",
                            resource.id()
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
