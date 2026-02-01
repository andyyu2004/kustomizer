use std::{collections::HashSet, fs::File, io::BufReader, sync::LazyLock};

use anyhow::Context as _;
use dashmap::DashMap;

use crate::{
    Located, PathExt, PathId,
    manifest::{Manifest, Patch, Target},
    patch::PatchResult,
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
        assert!(
            manifest.patches_strategic_merge.is_empty(),
            "patchesStrategicMerge should be translated to patches"
        );

        assert!(
            manifest.patches_json.is_empty(),
            "patchesJson6902 should be translated to patches"
        );

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
                let patch = yaml::from_reader::<JsonPatch>(BufReader::new(file))
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
    ) -> anyhow::Result<PatchResult> {
        let gvk = patch.gvk();
        match target {
            Some(target) => {
                if !target.matches(resource) {
                    return Ok(PatchResult::Retain);
                }
            }
            None => {
                // If no target is specified, match the patch gvk/name against the resource
                let matcher = GvkMatcher {
                    group: gvk.group.clone(),
                    version: gvk.version.clone(),
                    kind: gvk.kind.clone(),
                };

                if !resource.all_ids().any(|id| {
                    let mut gvk = resource.gvk().clone();
                    gvk.kind = id.kind.into();
                    matcher.matches(&gvk) && id.name == patch.name()
                }) {
                    return Ok(PatchResult::Retain);
                }
            }
        }

        resource.patch(patch)
    }
}

impl<A: Send + Sync, K: Send + Sync> Transformer for PatchTransformer<'_, A, K> {
    #[tracing::instrument(skip_all, name = "patch_transform")]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let mut to_delete = HashSet::new();

        for resource in resources.iter_mut() {
            let id = resource.id().clone();
            for patch in self.patches {
                match patch {
                    Patch::Json { patch, target } => {
                        if !target.matches(resource) {
                            continue;
                        }

                        json_patch(resource, patch)
                            .with_context(|| format!("applying JSON patch to resource `{id}`"))?;
                    }
                    Patch::StrategicMerge { patch, target } => {
                        let res = self
                            .apply_strategic_merge_patch(resource, patch.clone(), target)
                            .with_context(|| {
                                format!(
                                    "applying strategic merge patch to resource `{}`",
                                    resource.id()
                                )
                            })?;

                        match res {
                            PatchResult::Retain => (),
                            PatchResult::Delete => assert!(to_delete.insert(id.clone())),
                        }
                    }
                    Patch::OutOfLine { path, target } => {
                        let path = PathId::make(self.manifest.parent_path.join(path))?;
                        let patches =
                            Resource::load_many(path).context("loading out-of-line patches");

                        if let Ok(patches) = patches {
                            for patch in patches {
                                let res = self.apply_strategic_merge_patch(resource, patch, target)
                                    .with_context(|| {
                                        format!(
                                            "applying strategic merge patch from `{}` to resource `{}`",
                                            path.pretty(),
                                            resource.id()
                                        )
                                    })?;

                                match res {
                                    PatchResult::Retain => (),
                                    PatchResult::Delete => assert!(to_delete.insert(id.clone())),
                                }
                            }
                        } else {
                            let patch = self.load_json_patch(path)?;

                            let target = target.as_ref().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "patch target is required for json patch at `{}`",
                                    path.pretty()
                                )
                            })?;

                            if !target.matches(resource) {
                                continue;
                            }

                            json_patch(resource, &patch)?;
                        }
                    }
                }
            }
        }

        resources.retain(|id, _| !to_delete.contains(id));

        Ok(())
    }
}

fn json_patch(resource: &mut Resource, patch: &JsonPatch) -> anyhow::Result<()> {
    json_patch::patch(resource.root_raw_mut(), patch)
        .with_context(|| format!("applying JSON patch to resource `{}`", resource.id()))
}
