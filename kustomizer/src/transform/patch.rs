use anyhow::Context as _;

use crate::{
    Located, PathExt,
    manifest::{Manifest, Patch},
    resmap::ResourceMap,
    resource::Resource,
};

use super::Transformer;

pub struct PatchTransformer<'a, A, K> {
    manifest: &'a Located<Manifest<A, K>>,
    patches: &'a [Patch],
}

impl<'a, A, K> PatchTransformer<'a, A, K> {
    pub fn new(manifest: &'a Located<Manifest<A, K>>) -> Self {
        Self {
            patches: &manifest.patches,
            manifest,
        }
    }
}

#[async_trait::async_trait]
impl<A: Send + Sync, K: Send + Sync> Transformer for PatchTransformer<'_, A, K> {
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

                        let path = self.manifest.parent_path.join(path);
                        let patch = Resource::load(&path).with_context(|| {
                            format!(
                                "loading strategic merge patch from path `{}`",
                                path.pretty()
                            )
                        })?;

                        resource.patch(patch).with_context(|| {
                            format!(
                                "applying strategic merge patch from `{}` to resource `{}`",
                                path.pretty(),
                                resource.id()
                            )
                        })?;
                    }
                }
            }
        }

        Ok(())
    }
}
