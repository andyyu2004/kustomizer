// tmp lint relaxation
#![allow(dead_code, unused_imports)]

use std::{
    cell::RefCell,
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    Located, PathExt as _, PathId,
    generator::{ConfigMapGenerator, FunctionGenerator, Generator as _},
    load_component, load_file, load_kustomization, load_yaml,
    manifest::{
        Component, FunctionSpec, Generator, KeyValuePairSources, Kustomization, Manifest, Patch,
        Symbol,
    },
    reslist::ResourceList,
    resmap::ResourceMap,
    resource::{Gvk, ResId, Resource},
    transform::{AnnotationTransformer, LabelTransformer, NamespaceTransformer, Transformer},
};
use anyhow::{Context, bail};
use futures_util::future;
use indexmap::{IndexMap, map::Entry};
use tokio::sync::Mutex;

const KUSTOMIZE_FUNCTION_ANNOTATION: &str = "config.kubernetes.io/function";

#[derive(Debug, Default)]
pub struct Builder {
    // Filesystem caches to avoid re-reading files.
    components_cache: Mutex<IndexMap<PathId, Component>>,
    resources_cache: Mutex<IndexMap<PathId, Resource>>,
    strategic_merge_patches_cache: IndexMap<PathId, serde_yaml::Value>,
    key_value_files_cache: IndexMap<PathId, Box<str>>,
}

impl Builder {
    pub async fn build_kustomization(
        &self,
        kustomization: &Located<Kustomization>,
    ) -> anyhow::Result<ResourceMap> {
        self.build(kustomization).await
    }

    #[tracing::instrument(skip_all, fields(path = %kustomization.path.pretty()))]
    #[async_recursion::async_recursion]
    #[allow(clippy::multiple_bound_locations)]
    pub async fn build<A: Symbol, K: Symbol>(
        &self,
        kustomization: &Located<Manifest<A, K>>,
    ) -> anyhow::Result<ResourceMap> {
        let mut resources = self.build_kustomization_base(kustomization).await?;

        for path in &kustomization.generators {
            let path = PathId::make(kustomization.parent_path.join(path))?;
            let workdir = path.parent().unwrap();
            let generator_spec = load_yaml::<Resource>(path)
                .with_context(|| format!("loading generator spec from {}", path.pretty()))?;
            let function_spec = match generator_spec.metadata().annotations() {
                Some(annotations) => match annotations.function_spec() {
                    Ok(Some(spec)) => spec,
                    Ok(None) => bail!(
                        "generator spec at `{}` is missing `{KUSTOMIZE_FUNCTION_ANNOTATION}` annotation",
                        path.pretty()
                    ),
                    Err(err) => bail!("failed parsing function spec: {err}"),
                },
                None => bail!(
                    "generator spec at `{}` is missing `{KUSTOMIZE_FUNCTION_ANNOTATION}` annotation",
                    path.pretty()
                ),
            };

            let generated = FunctionGenerator::new(function_spec)
                .generate(workdir, &ResourceList::new([generator_spec]))
                .await
                .with_context(|| {
                    format!(
                        "failed generating resources from function spec at {}",
                        path.pretty()
                    )
                })?;

            resources.extend(generated).with_context(|| {
                format!(
                    "failure merging resources from generator at `{}`",
                    path.pretty()
                )
            })?;
        }

        let configmaps = ConfigMapGenerator::new(
            &kustomization.config_map_generators,
            &kustomization.generator_options,
        )
        .generate(&kustomization.parent_path, &ResourceList::new([]))
        .await?;
        resources.extend(configmaps).with_context(|| {
            format!(
                "failure merging resources from configmap generators in `{}`",
                kustomization.path.pretty()
            )
        })?;

        for component in &kustomization.components {
            let component = load_component(kustomization.parent_path.join(component))
                .with_context(|| format!("loading component `{}`", component.pretty()))?;
            let built = self.build(&component).await?;
            resources.merge(built).with_context(|| {
                format!(
                    "failure merging resources from component `{}`",
                    component.path.pretty()
                )
            })?;
        }

        AnnotationTransformer(&kustomization.common_annotations).transform(&mut resources);
        LabelTransformer(&kustomization.labels).transform(&mut resources);
        if let Some(namespace) = &kustomization.namespace {
            NamespaceTransformer(namespace.clone()).transform(&mut resources);
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
        //
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

    async fn build_kustomization_base<A, K>(
        &self,
        kustomization: &Located<Manifest<A, K>>,
    ) -> anyhow::Result<ResourceMap> {
        let mut resmap = ResourceMap::default();

        let resources =
            future::try_join_all(kustomization.resources.iter().cloned().map(|path| async {
                let built = self.build_resource(kustomization, &path).await?;
                anyhow::Ok((path, built))
            }))
            .await?;

        for (path, resource) in resources {
            match resource {
                either::Either::Left(res) => resmap.insert(res),
                either::Either::Right(base) => resmap.merge(base),
            }
            .with_context(|| format!("failure merging resources from `{}`", path.pretty()))?
        }

        Ok(resmap)
    }

    #[tracing::instrument(skip_all, fields(path = %kustomization.path.pretty(), resource_path = %path.pretty()))]
    async fn build_resource<A, K>(
        &self,
        kustomization: &Located<Manifest<A, K>>,
        path: &Path,
    ) -> anyhow::Result<either::Either<Resource, ResourceMap>> {
        let path = PathId::make(kustomization.parent_path.join(path))?;

        let metadata = std::fs::metadata(path)
            .with_context(|| format!("reading metadata for resource {}", path.pretty()))?;

        if metadata.is_symlink() {
            bail!("symlinks are not implemented: {}", path.pretty());
        } else if metadata.is_file() {
            let res = match self.resources_cache.lock().await.entry(path) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let res = load_yaml::<Resource>(path)
                        .with_context(|| format!("loading resource {}", path.pretty()))?;
                    entry.insert(res)
                }
            }
            .clone();

            Ok(either::Either::Left(res))
        } else {
            let kustomization = load_kustomization(path)
                .with_context(|| format!("loading kustomization resource {}", path.pretty()))?;

            let base = self
                .build(&kustomization)
                .await
                .with_context(|| format!("building kustomization resource {}", path.pretty()))?;

            Ok(either::Either::Right(base))
        }
    }
}
