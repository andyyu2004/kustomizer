// tmp lint relaxation
#![allow(dead_code, unused_imports)]

use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    Located, PathExt as _, PathId,
    generator::{FunctionGenerator, Generator as _},
    load_file, load_kustomization, load_yaml,
    manifest::{
        Component, FunctionSpec, Generator, KeyValuePairSources, Kustomization, Manifest, Patch,
    },
    reslist::ResourceList,
    resmap::ResourceMap,
    resource::Resource,
    transform::{AnnotationTransformer, Transformer},
};
use anyhow::{Context, bail};
use indexmap::{IndexMap, map::Entry};

const KUSTOMIZE_FUNCTION_ANNOTATION: &str = "config.kubernetes.io/function";

#[derive(Debug, Default)]
pub struct Builder {
    // Filesystem caches to avoid re-reading files.
    components_cache: IndexMap<PathId, Component>,
    resources_cache: IndexMap<PathId, Resource>,
    strategic_merge_patches_cache: IndexMap<PathId, serde_yaml::Value>,
    key_value_files_cache: IndexMap<PathId, Box<str>>,
}

impl Builder {
    #[tracing::instrument(skip_all, fields(path = %kustomization.path.pretty()))]
    pub async fn build(
        mut self,
        kustomization: &Located<Kustomization>,
        out: &mut dyn Write,
    ) -> anyhow::Result<()> {
        // self.gather(kustomization)?;
        let output = self.build_kustomization(kustomization).await?;

        for resource in output.iter() {
            if output.len() > 1 {
                writeln!(out, "---")?;
            }
            serde_yaml::to_writer(&mut *out, resource)?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(path = %kustomization.path.pretty()))]
    async fn build_kustomization(
        &mut self,
        kustomization: &Located<Kustomization>,
    ) -> anyhow::Result<ResourceMap> {
        let mut resources = self.build_kustomization_base(kustomization).await?;

        for path in &kustomization.generators {
            let path = PathId::make(kustomization.parent_path.join(path))?;
            let workdir = path.parent().unwrap();
            let generator_spec = load_yaml::<Resource>(path)
                .with_context(|| format!("loading generator spec from {}", path.pretty()))?;
            let function_spec_str = generator_spec
                .metadata
                .annotations
                .get(KUSTOMIZE_FUNCTION_ANNOTATION)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "generator spec at `{}` is missing `{KUSTOMIZE_FUNCTION_ANNOTATION}` annotation",
                        path.pretty()
                    )
                })?;

            // Do a strange dance where to we convert to JSON first to avoid serde_yaml's !Tag based enum deserialization format.
            let json = serde_yaml::from_str::<serde_json::Value>(function_spec_str)?;
            let spec = serde_json::from_value::<FunctionSpec>(json)
                .with_context(|| format!("parsing function spec from {}", path.pretty()))?;

            let generated = FunctionGenerator::new(spec)
                .generate(workdir, &ResourceList::new([generator_spec]))
                .with_context(|| {
                    format!(
                        "generating resources from function spec at {}",
                        path.pretty()
                    )
                })?;

            if let Err(id) = resources.merge(generated) {
                bail!(
                    "merging resources from generator `{}`: may not add resource with an already registered id `{id}`",
                    path.pretty(),
                );
            }
        }

        AnnotationTransformer(&kustomization.common_annotations).transform(&mut resources);

        for resource in resources.iter_mut() {
            for label in &kustomization.labels {
                for (key, value) in &label.pairs {
                    // `kustomization.labels` takes precedence over resource metadata labels
                    resource.metadata.labels.insert(key.clone(), value.clone());
                }
            }

            if let Some(namespace) = &kustomization.namespace {
                resource.metadata.namespace = Some(namespace.clone());
            }
        }

        // if !kustomization.patches.is_empty() {
        //     bail!("patches are not implemented");
        // }
        //
        // if !kustomization.transformers.is_empty() {
        //     bail!("transformers are not implemented");
        // }
        //
        //
        // if !kustomization.replicas.is_empty() {
        //     bail!("images are not implemented");
        // }
        //
        // if !kustomization.components.is_empty() {
        //     bail!("components are not implemented");
        // }
        //
        // if !kustomization.config_map_generators.is_empty() {
        //     bail!("config map generators are not implemented");
        // }
        //
        // if kustomization.name_prefix.is_some() {
        //     bail!("name prefix is not implemented");
        // }
        //
        // if kustomization.name_suffix.is_some() {
        //     bail!("name suffix is not implemented");
        // }

        Ok(resources)
    }

    async fn build_kustomization_base(
        &mut self,
        kustomization: &Located<Kustomization>,
    ) -> anyhow::Result<ResourceMap> {
        let mut resources = ResourceMap::default();

        for path in &kustomization.resources {
            let path = PathId::make(kustomization.parent_path.join(path))?;

            let metadata = std::fs::metadata(path)
                .with_context(|| format!("reading metadata for resource {}", path.pretty()))?;

            if metadata.is_symlink() {
                bail!("symlinks are not implemented: {}", path.pretty());
            } else if metadata.is_file() {
                let res = match self.resources_cache.entry(path) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => {
                        let res = load_yaml::<Resource>(path)
                            .with_context(|| format!("loading resource {}", path.pretty()))?;
                        entry.insert(res)
                    }
                };

                if resources.insert(res.clone()).is_some() {
                    bail!(
                        "merging resources from `{}`: may not add resource with an already registered id `{}`",
                        path.pretty(),
                        res.id
                    );
                }
            } else {
                let kustomization = load_kustomization(path)
                    .with_context(|| format!("loading kustomization resource {}", path.pretty()))?;

                let base = Box::pin(self.build_kustomization(&kustomization))
                    .await
                    .with_context(|| {
                        format!("building kustomization resource {}", path.pretty())
                    })?;

                if let Err(res_id) = resources.merge(base) {
                    bail!(
                        "merging resources from `{}`: may not add resource with an already registered id `{res_id}`",
                        path.pretty(),
                    );
                }
            }
        }

        Ok(resources)
    }
}
