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
    generator::{ConfigMapGenerator, Generator as _},
    load_component, load_file, load_kustomization, load_yaml,
    manifest::{
        Component, FunctionSpec, Generator, KeyValuePairSources, Kustomization, Manifest, Patch,
        Symbol,
    },
    plugin::FunctionPlugin,
    reslist::ResourceList,
    resmap::ResourceMap,
    resource::{Gvk, ResId, Resource},
    transform::{
        AnnotationTransformer, ImageTagTransformer, LabelTransformer, NamespaceTransformer,
        Transformer,
    },
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

            if let Some(annotations) = generator_spec.annotations()
                && annotations.has(KUSTOMIZE_FUNCTION_ANNOTATION)
            {
                let function_spec = annotations
                    .function_spec()
                    .with_context(|| {
                        format!(
                            "failed parsing function spec from generator spec at `{}`",
                            path.pretty()
                        )
                    })?
                    .unwrap();
                let generated = FunctionPlugin::new(function_spec)
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
            } else {
                bail!(
                    "only custom generators with the `{KUSTOMIZE_FUNCTION_ANNOTATION}` annotation are supported `{}`, got {}",
                    path.pretty(),
                    generator_spec.id()
                );
            }
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
            // FIXME This isn't right, the transformers patches of the components need to be applied to the base as well
            let built = self.build(&component).await?;
            resources.merge(built).with_context(|| {
                format!(
                    "failure merging resources from component `{}`",
                    component.path.pretty()
                )
            })?;
        }

        LabelTransformer(&kustomization.labels)
            .transform(&mut resources)
            .await?;
        AnnotationTransformer(&kustomization.common_annotations)
            .transform(&mut resources)
            .await?;
        if let Some(namespace) = &kustomization.namespace {
            NamespaceTransformer(namespace.clone())
                .transform(&mut resources)
                .await?;
        }

        // if !kustomization.patches.is_empty() {
        //     bail!("patches are not implemented");
        // }
        //

        for path in &kustomization.transformers {
            let path = PathId::make(kustomization.parent_path.join(path))?;
            let transformer_spec = load_yaml::<Resource>(path)
                .with_context(|| format!("loading transformer spec from {}", path.pretty()))?;

            if let Some(annotations) = transformer_spec.annotations()
                && annotations.has(KUSTOMIZE_FUNCTION_ANNOTATION)
            {
                let function_spec = annotations.function_spec()?.unwrap();
                FunctionPlugin::new(function_spec)
                    .transform(&mut resources)
                    .await
                    .with_context(|| {
                        format!(
                            "failed transforming resources with function spec at `{}`",
                            path.pretty()
                        )
                    })?;
            } else if transformer_spec.api_version() == "builtin" {
                match transformer_spec.kind().as_str() {
                    "ImageTagTransformer" => {
                        serde_yaml::from_value::<ImageTagTransformer>(serde_yaml::Value::Mapping(
                            transformer_spec.root().clone(),
                        ))
                        .with_context(|| {
                            format!("failed parsing ImageTagTransformer at `{}`", path.pretty())
                        })?
                        .transform(&mut resources)
                        .await?
                    }
                    _ => bail!(
                        "unknown builtin transformer kind `{}` at `{}`",
                        transformer_spec.kind(),
                        path.pretty()
                    ),
                }
            } else {
                bail!(
                    "only builtin or custom transformers with `{KUSTOMIZE_FUNCTION_ANNOTATION}` annotation are supported `{}`, got {}",
                    path.pretty(),
                    transformer_spec.id()
                );
            }
        }

        //
        //
        // if !kustomization.replicas.is_empty() {
        //     bail!("images are not implemented");
        // }
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
