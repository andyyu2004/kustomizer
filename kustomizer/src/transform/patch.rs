use anyhow::Context as _;

use crate::{
    Located, PathExt,
    manifest::{Manifest, Patch},
    resmap::ResourceMap,
    resource::{GvkMatcher, Resource},
};

use super::Transformer;
use json_patch::Patch as JsonPatch;

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

impl<A: Send + Sync, K: Send + Sync> Transformer for PatchTransformer<'_, A, K> {
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
                        let path = self.manifest.parent_path.join(path);
                        let patch = Resource::load(&path).with_context(|| {
                            format!("loading resource from path `{}`", path.pretty())
                        });

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

                            resource.patch(patch).with_context(|| {
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

                            let file = std::fs::File::open(&path).with_context(|| {
                                format!("opening JSON patch file at path `{}`", path.pretty())
                            })?;

                            // TODO avoid reading the file multiple times
                            let patch = serde_yaml::from_reader::<_, JsonPatch>(file)
                                .with_context(|| {
                                    format!("parsing JSON patch from file `{}`", path.pretty())
                                })?;
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
