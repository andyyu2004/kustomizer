use std::{collections::HashMap, fs::File};

use anyhow::Context as _;

use crate::{
    Located, PathExt, PathId,
    manifest::{Manifest, Patch},
    resmap::ResourceMap,
    resource::{GvkMatcher, Resource},
};

use super::Transformer;
use json_patch::Patch as JsonPatch;

pub struct PatchTransformer<'a, A, K> {
    manifest: &'a Located<Manifest<A, K>>,
    patches: &'a [Patch],
    res_cache: HashMap<PathId, Resource>,
    patch_cache: HashMap<PathId, JsonPatch>,
}

impl<'a, A, K> PatchTransformer<'a, A, K> {
    pub fn new(manifest: &'a Located<Manifest<A, K>>) -> Self {
        Self {
            patches: &manifest.patches,
            manifest,
            res_cache: Default::default(),
            patch_cache: Default::default(),
        }
    }

    fn load_resource(&mut self, path: PathId) -> anyhow::Result<&Resource> {
        match self.res_cache.entry(path) {
            std::collections::hash_map::Entry::Occupied(e) => Ok(e.into_mut()),
            std::collections::hash_map::Entry::Vacant(e) => {
                let resource = Resource::load(path)
                    .with_context(|| format!("loading resource from path `{}`", path.pretty()))?;
                Ok(e.insert(resource))
            }
        }
    }

    fn load_json_patch(&self, path: PathId) -> anyhow::Result<JsonPatch> {
        match self.patch_cache.get(&path) {
            Some(patch) => Ok(patch.clone()),
            None => {
                let file = File::open(path).with_context(|| {
                    format!("opening JSON patch file at path `{}`", path.pretty())
                })?;
                let patch = serde_yaml::from_reader::<_, JsonPatch>(file)
                    .with_context(|| format!("parsing JSON patch from file `{}`", path.pretty()))?;
                Ok(patch)
            }
        }
    }
}

impl<A: Send + Sync, K: Send + Sync> Transformer for PatchTransformer<'_, A, K> {
    #[tracing::instrument(skip_all, name = "patch_transform")]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        // We don't know what the patches will do, it may patch identity fields.
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
                        let _ = (patch, target);
                        todo!("inline strategic merge")
                    }
                    Patch::OutOfLine { path, target } => {
                        let path = PathId::make(self.manifest.parent_path.join(path))?;
                        let patch = self.load_resource(path);

                        if let Ok(patch) = patch {
                            let gvk = patch.gvk();
                            let matcher = GvkMatcher {
                                group: gvk.group.clone(),
                                version: gvk.version.clone(),
                                kind: gvk.kind.clone(),
                            };
                            match target {
                                Some(target) => {
                                    if !target.matches(&resource) {
                                        continue;
                                    }
                                }
                                None => {
                                    // If no target is specified, match the patch gvk/name against the resource
                                    if !matcher.matches(resource.gvk())
                                        || patch.name() != resource.name()
                                    {
                                        continue;
                                    }
                                }
                            }

                            resource.patch(patch.clone()).with_context(|| {
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
    let mut raw = serde_json::to_value(&resource)?;
    json_patch::patch(&mut raw, patch)
        .with_context(|| format!("applying JSON patch to resource `{}`", resource.id()))?;
    serde_json::from_value(raw).map_err(Into::into)
}
