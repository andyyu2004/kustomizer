use std::{io::Write, path::Path};

use crate::{
    Located, PathId, load_file, load_kustomization, load_yaml,
    manifest::{Component, Generator, KeyValuePairSources, Kustomization, Manifest, Patch},
    resmap::ResourceMap,
    resource::Resource,
};
use anyhow::Context;
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct Builder {
    /// Maps from a kustomization directory to its kustomization file path.
    kustomizations: IndexMap<PathId, Kustomization>,
    components: IndexMap<PathId, Component>,
    resources: IndexMap<PathId, Resource>,
    strategic_merge_patches: IndexMap<PathId, serde_yaml::Value>,
    key_value_files: IndexMap<PathId, Box<str>>,
    output: ResourceMap,
}

impl Builder {
    #[tracing::instrument(skip_all, fields(path = %kustomization.path.display()))]
    pub fn build(
        mut self,
        kustomization: &Located<Kustomization>,
        out: &mut dyn Write,
    ) -> anyhow::Result<()> {
        assert!(
            self.kustomizations
                .insert(kustomization.path, kustomization.value.clone())
                .is_none()
        );

        self.gather(kustomization)?;
        self.build_root(kustomization)?;

        for resource in self.output.iter() {
            if self.output.len() > 1 {
                writeln!(out, "---")?;
            }
            serde_yaml::to_writer(&mut *out, resource)?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn build_root(&mut self, kustomization: &Located<Kustomization>) -> anyhow::Result<()> {
        for resource in self.resources.values() {
            if let Some(old) = self.output.insert(resource.clone()) {
                todo!("handle duplicate resource: {}", old.id);
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn gather_resources<'a>(
        &mut self,
        base_path: &Path,
        resources: impl Iterator<Item = &'a Path>,
    ) -> anyhow::Result<()> {
        for path in resources {
            let path = base_path.join(path);
            let path = PathId::make(&path)
                .with_context(|| format!("canonicalizing resource path {}", path.display()))?;

            if self.resources.contains_key(&path) {
                continue;
            }

            // TODO handle symlinks
            let metadata = std::fs::metadata(path)?;
            if metadata.is_file() {
                let resource = crate::load_yaml(path)
                    .with_context(|| format!("loading resource {}", path.display()))?;
                assert!(self.resources.insert(path, resource).is_none());
            } else if metadata.is_dir() {
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
                self.gather(&kustomization)?;
                assert_eq!(
                    self.kustomizations
                        .insert(kustomization.path, kustomization.value),
                    Some(Default::default()),
                );
            } else if metadata.is_symlink() {
                return Err(anyhow::anyhow!(
                    "symlinks are not implemented: {}",
                    path.display()
                ));
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn gather_patches<'a>(
        &mut self,
        base_path: &Path,
        patches: impl Iterator<Item = &'a Patch>,
    ) -> anyhow::Result<()> {
        for patch in patches {
            match patch {
                Patch::Json { .. } => {}
                Patch::StrategicMerge { path, .. } => {
                    let path = PathId::make(base_path.join(path)).with_context(|| {
                        format!(
                            "canonicalizing strategic merge patch path {}",
                            path.display()
                        )
                    })?;

                    if self.strategic_merge_patches.contains_key(&path) {
                        continue;
                    }

                    let patch = load_yaml::<serde_yaml::Value>(path).with_context(|| {
                        format!("loading strategic merge patch {}", path.display())
                    })?;

                    assert!(self.strategic_merge_patches.insert(path, patch).is_none());
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn gather_components<'a>(
        &mut self,
        base_path: &Path,
        components: impl Iterator<Item = &'a Path>,
    ) -> anyhow::Result<()> {
        for path in components {
            let path = PathId::make(base_path.join(path))
                .with_context(|| format!("canonicalizing component path {}", path.display()))?;

            if self.components.contains_key(&path) {
                continue;
            }

            let component = crate::load_component(path)
                .with_context(|| format!("loading component {}", path.display()))?;

            // Insert a placeholder to avoid cycles causing overflow. TODO detect cycles and report them.
            assert!(self.components.insert(path, Component::default()).is_none());
            self.gather(&component)?;
            assert_eq!(
                self.components.insert(path, component.value),
                Some(Component::default())
            );
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn gather_configmap_generators<'a>(
        &mut self,
        base_path: &Path,
        generators: impl Iterator<Item = &'a Generator>,
    ) -> anyhow::Result<()> {
        for generator in generators {
            self.gather_key_value_pair_sources(base_path, &generator.sources)?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn gather_key_value_pair_sources(
        &mut self,
        base_path: &Path,
        sources: &KeyValuePairSources,
    ) -> anyhow::Result<()> {
        for file in &sources.files {
            let path = PathId::make(base_path.join(&file.value))?;
            let data = load_file(path)
                .with_context(|| format!("loading key-value file {}", file.value))?;
            self.key_value_files.insert(path, data.into_boxed_str());
        }
        Ok(())
    }

    // gather all referenced files and read them into memory.
    #[tracing::instrument(skip_all, fields(path = %manifest.path.display()))]
    fn gather<A, K>(&mut self, manifest: &Located<Manifest<A, K>>) -> anyhow::Result<()> {
        let base_path = manifest.parent_path;

        self.gather_resources(&base_path, manifest.resources.iter().map(|p| p.as_path()))
            .with_context(|| {
                format!(
                    "gathering resources from kustomization at {}",
                    manifest.path.display()
                )
            })?;

        self.gather_patches(&base_path, manifest.patches.iter())
            .with_context(|| {
                format!(
                    "gathering patches from kustomization at {}",
                    manifest.path.display()
                )
            })?;

        self.gather_components(&base_path, manifest.components.iter().map(|p| p.as_path()))
            .with_context(|| {
                format!(
                    "gathering components from kustomization at {}",
                    manifest.path.display()
                )
            })?;

        self.gather_configmap_generators(&base_path, manifest.config_map_generators.iter())
            .with_context(|| {
                format!(
                    "gathering configmap generators from kustomization at {}",
                    manifest.path.display()
                )
            })?;

        // TODO generators and transformers

        Ok(())
    }
}
