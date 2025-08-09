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
    load_component, load_kustomization,
    manifest::{
        Component, FunctionSpec, Generator, KeyValuePairSources, Kustomization, Manifest, Patch,
        Symbol,
    },
    plugin::FunctionPlugin,
    reslist::ResourceList,
    resmap::ResourceMap,
    resource::{Gvk, ResId, Resource},
    transform::{
        AnnotationTransformer, CleanupTransformer, ImageTagTransformer, LabelTransformer,
        NameTransformer, NamespaceTransformer, PatchTransformer, ReplicaTransformer, Transformer,
    },
};
use anyhow::{Context, bail};
use compact_str::format_compact;
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
    pub async fn build_kust(
        &self,
        kustomization: &Located<Kustomization>,
    ) -> anyhow::Result<ResourceMap> {
        self.build(Default::default(), kustomization).await
    }

    #[tracing::instrument(skip_all, fields(path = %kustomization.path.pretty()))]
    #[async_recursion::async_recursion]
    #[allow(clippy::multiple_bound_locations)]
    async fn build<A: Symbol, K: Symbol>(
        &self,
        resmap: ResourceMap,
        kustomization: &Located<Manifest<A, K>>,
    ) -> anyhow::Result<ResourceMap> {
        let mut resmap = self.build_kustomization_base(resmap, kustomization).await?;

        for path in &kustomization.generators {
            let path = PathId::make(kustomization.parent_path.join(path))?;
            let workdir = path.parent().unwrap();
            let generator_spec = Resource::load(path)
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

                resmap.extend(generated).with_context(|| {
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
        resmap.extend(configmaps).with_context(|| {
            format!(
                "failure merging resources from configmap generators in `{}`",
                kustomization.path.pretty()
            )
        })?;

        LabelTransformer(&kustomization.labels)
            .transform(&mut resmap)
            .await?;
        AnnotationTransformer(&kustomization.common_annotations)
            .transform(&mut resmap)
            .await?;
        if let Some(namespace) = &kustomization.namespace {
            NamespaceTransformer(namespace.clone())
                .transform(&mut resmap)
                .await?;
        }

        PatchTransformer::new(kustomization)
            .transform(&mut resmap)
            .await?;

        ReplicaTransformer::new(&kustomization.replicas)
            .transform(&mut resmap)
            .await?;

        match (&kustomization.name_prefix, &kustomization.name_suffix) {
            (None, None) => {}
            (Some(prefix), None) => {
                NameTransformer::new(|name| format_compact!("{prefix}{name}"))
                    .transform(&mut resmap)
                    .await?;
            }
            (None, Some(suffix)) => {
                NameTransformer::new(|name| format_compact!("{name}{suffix}"))
                    .transform(&mut resmap)
                    .await?
            }
            (Some(prefix), Some(suffix)) => {
                NameTransformer::new(|name| format_compact!("{prefix}{name}{suffix}"))
                    .transform(&mut resmap)
                    .await?;
            }
        };

        for path in &kustomization.transformers {
            let path = PathId::make(kustomization.parent_path.join(path))?;
            let transformer_spec = Resource::load(path)
                .with_context(|| format!("loading transformer spec from {}", path.pretty()))?;

            if let Some(annotations) = transformer_spec.annotations()
                && annotations.has(KUSTOMIZE_FUNCTION_ANNOTATION)
            {
                let function_spec = annotations.function_spec()?.unwrap();
                FunctionPlugin::new(function_spec)
                    .transform(&mut resmap)
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
                        serde_json::from_value::<ImageTagTransformer>(serde_json::Value::Object(
                            transformer_spec.root().clone(),
                        ))
                        .with_context(|| {
                            format!("failed parsing ImageTagTransformer at `{}`", path.pretty())
                        })?
                        .transform(&mut resmap)
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

        for component in &kustomization.components {
            let component = load_component(kustomization.parent_path.join(component))
                .with_context(|| format!("loading component `{}`", component.pretty()))?;
            resmap = self.build(resmap, &component).await?;
        }

        CleanupTransformer::default().transform(&mut resmap).await?;

        Ok(resmap)
    }

    async fn build_kustomization_base<A, K>(
        &self,
        mut resmap: ResourceMap,
        kustomization: &Located<Manifest<A, K>>,
    ) -> anyhow::Result<ResourceMap> {
        let resources =
            future::try_join_all(kustomization.resources.iter().cloned().map(|path| async {
                let built = self.build_resource(kustomization, &path).await?;
                anyhow::Ok((path, built))
            }))
            .await?;

        for (path, resource) in resources {
            match resource {
                either::Either::Left(res) => resmap.insert(res),
                either::Either::Right(rs) => resmap.merge(rs),
            }
            .with_context(|| {
                format!(
                    "failure merging resources from `{}` into `{}`",
                    path.pretty(),
                    kustomization.path.pretty()
                )
            })?
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
                    let res = Resource::load(path)
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
                .build_kust(&kustomization)
                .await
                .with_context(|| format!("building kustomization resource {}", path.pretty()))?;

            Ok(either::Either::Right(base))
        }
    }
}
