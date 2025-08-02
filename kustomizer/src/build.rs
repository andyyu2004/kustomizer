use std::collections::HashMap;

use crate::{
    Located, PathId, load_kustomization, load_resource,
    manifest::{Component, Kustomization, Manifest, PathOrInlinePatch},
};
use anyhow::Context;

#[derive(Debug, Default)]
pub struct Builder {
    kustomizations: HashMap<PathId, Kustomization>,
    components: HashMap<PathId, Component>,
    resource: HashMap<PathId, serde_yaml::Value>,
    json_patches: HashMap<PathId, json_patch::Patch>,
    strategic_merge_patches: HashMap<PathId, serde_yaml::Value>,
}

impl Builder {
    pub async fn build(mut self, kustomization: &Located<Kustomization>) -> anyhow::Result<()> {
        self.gather(kustomization).await?;
        dbg!(&self);
        Ok(())
    }

    async fn gather(&mut self, kustomization: &Located<Kustomization>) -> anyhow::Result<()> {
        assert!(
            self.kustomizations
                .insert(kustomization.path, kustomization.value.clone())
                .is_none()
        );
        self.traverse(kustomization)?;

        Ok(())
    }

    fn traverse<A, K>(&mut self, manifest: &Located<Manifest<A, K>>) -> anyhow::Result<()> {
        let base_path = manifest
            .path
            .parent()
            .expect("this is a file so it has a parent")
            .canonicalize()?;

        for path in &manifest.resources {
            let path = PathId::make(base_path.join(path))
                .with_context(|| format!("canonicalizing resource path {}", path.display()))?;

            if self.resource.contains_key(&path) {
                continue;
            }

            if path.is_file() {
                let resource = crate::load_resource::<serde_yaml::Value>(path)
                    .with_context(|| format!("loading resource {}", path.display()))?;
                assert!(self.resource.insert(path, resource).is_none());
            } else {
                let kustomization = load_kustomization(path).with_context(|| {
                    format!("loading kustomization resource {}", path.display())
                })?;

                if self.kustomizations.contains_key(&kustomization.path) {
                    continue;
                }

                assert!(
                    self.kustomizations
                        .insert(kustomization.path, Default::default())
                        .is_none()
                );
                self.traverse(&kustomization)?;
                assert_eq!(
                    self.kustomizations
                        .insert(kustomization.path, kustomization.value),
                    Some(Default::default()),
                );
            }
        }

        for patch in &manifest.patches {
            match patch {
                crate::manifest::Patch::Json { patch, target: _ } => match patch {
                    PathOrInlinePatch::Inline(_) => {}
                    PathOrInlinePatch::Path(path) => {
                        let path = PathId::make(base_path.join(path)).with_context(|| {
                            format!("canonicalizing json patch path {}", path.display())
                        })?;
                        if self.json_patches.contains_key(&path) {
                            continue;
                        }

                        let patch = load_resource(path)
                            .with_context(|| format!("loading json patch {}", path.display()))?;

                        assert!(self.json_patches.insert(path, patch).is_none());
                    }
                },
                crate::manifest::Patch::StrategicMerge { path } => {
                    let path = PathId::make(base_path.join(path)).with_context(|| {
                        format!(
                            "canonicalizing strategic merge patch path {}",
                            path.display()
                        )
                    })?;
                    if self.json_patches.contains_key(&path) {
                        continue;
                    }

                    let patch = load_resource::<serde_yaml::Value>(path).with_context(|| {
                        format!("loading strategic merge patch {}", path.display())
                    })?;

                    assert!(self.strategic_merge_patches.insert(path, patch).is_none());
                }
            }
        }

        for path in &manifest.components {
            let path = PathId::make(base_path.join(path))
                .with_context(|| format!("canonicalizing component path {}", path.display()))?;

            if self.components.contains_key(&path) {
                continue;
            }

            let component = crate::load_component(path)
                .with_context(|| format!("loading component {}", path.display()))?;

            // Insert a placeholder to avoid cycles causing overflow. TODO detect cycles and report them.
            assert!(self.components.insert(path, Component::default()).is_none());
            self.traverse(&component)?;
            assert_eq!(
                self.components.insert(path, component.value),
                Some(Component::default())
            );
        }

        Ok(())
    }
}
