use std::{fs::File, sync::LazyLock};

use anyhow::Context as _;
use dashmap::DashMap;

use crate::{
    Located, PathExt, PathId,
    manifest::{Manifest, Patch, Target},
    resmap::ResourceMap,
    resource::{GvkMatcher, Resource},
    yaml,
};

use super::Transformer;
use json_patch::Patch as JsonPatch;

static PATCH_CACHE: LazyLock<DashMap<PathId, JsonPatch>> = LazyLock::new(Default::default);

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

    fn load_json_patch(&self, path: PathId) -> anyhow::Result<JsonPatch> {
        match PATCH_CACHE.get(&path) {
            Some(patch) => Ok(patch.clone()),
            None => {
                let file = File::open(path).with_context(|| {
                    format!("opening JSON patch file at path `{}`", path.pretty())
                })?;
                let patch = yaml::from_reader::<JsonPatch>(file)
                    .with_context(|| format!("parsing JSON patch from file `{}`", path.pretty()))?;
                Ok(patch)
            }
        }
    }

    fn apply_strategic_merge_patch(
        &self,
        resource: &mut Resource,
        patch: Resource,
        target: &Option<Target>,
    ) -> anyhow::Result<()> {
        let gvk = patch.gvk();
        match target {
            Some(target) => {
                if !target.matches(resource) {
                    return Ok(());
                }
            }
            None => {
                // If no target is specified, match the patch gvk/name against the resource
                let matcher = GvkMatcher {
                    group: gvk.group.clone(),
                    version: gvk.version.clone(),
                    kind: gvk.kind.clone(),
                };

                // FIXME match all ids
                if !matcher.matches(resource.gvk()) || resource.name() != patch.name() {
                    return Ok(());
                }
            }
        }

        resource.patch(patch.clone())
    }
}

impl<A: Send + Sync, K: Send + Sync> Transformer for PatchTransformer<'_, A, K> {
    #[tracing::instrument(skip_all, name = "patch_transform")]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let mut out = ResourceMap::with_capacity(resources.len());
        for mut resource in std::mem::take(resources) {
            let id = resource.id().clone();
            for patch in self.patches {
                match patch {
                    Patch::Json { patch, target } => {
                        if !target.matches(&resource) {
                            continue;
                        }

                        resource = json_patch(resource, patch)
                            .with_context(|| format!("applying JSON patch to resource `{id}`"))?;
                    }
                    Patch::StrategicMerge { patch, target } => {
                        self.apply_strategic_merge_patch(&mut resource, patch.clone(), target)
                            .with_context(|| {
                                format!(
                                    "applying strategic merge patch to resource `{}`",
                                    resource.id()
                                )
                            })?;
                    }
                    Patch::OutOfLine { path, target } => {
                        let path = PathId::make(self.manifest.parent_path.join(path))?;
                        let patch = Resource::load(path);

                        if let Ok(patch) = patch {
                            self.apply_strategic_merge_patch(&mut resource, patch, target)
                                .with_context(|| {
                                    format!(
                                        "applying strategic merge patch from `{}` to resource `{}`",
                                        path.pretty(),
                                        resource.id()
                                    )
                                })?;
                        } else {
                            let target = target.as_ref().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "patch target is required for json patch at `{}`",
                                    path.pretty()
                                )
                            })?;

                            if !target.matches(&resource) {
                                continue;
                            }

                            let patch = self.load_json_patch(path)?;
                            resource = json_patch(resource, &patch)?;
                        }
                    }
                }
            }

            out.insert(resource)
                .with_context(|| format!("adding patched resource `{id}`"))?;
        }

        *resources = out;

        Ok(())
    }
}

fn json_patch(resource: Resource, patch: &JsonPatch) -> anyhow::Result<Resource> {
    let mut raw = json::to_value(&resource)?;
    json_patch::patch(&mut raw, patch)
        .with_context(|| format!("applying JSON patch to resource `{}`", resource.id()))?;
    json::from_value(raw).map_err(Into::into)
}
