mod build;
pub mod dbg;
mod fieldspec;
mod generator;
mod intern;
pub mod manifest;
mod patch;
mod plugin;
mod reslist;
mod resmap;
mod resource;
mod selector;
mod serde_ex;
mod transform;
pub mod yaml;

use core::fmt;
use std::{ffi::OsStr, io::BufReader, mem, ops::Deref, path::Path};

use anyhow::{Context, bail};

pub use self::intern::PathId;
pub use self::resmap::ResourceMap;

use self::{
    manifest::{Component, Kustomization, Label, Manifest, Patch, Symbol, kind},
    patch::openapi,
    resource::Resource,
};

pub async fn build(path: impl AsRef<Path>) -> anyhow::Result<ResourceMap> {
    let kustomization = load_kustomization(path)?;
    build::Builder::default().build_kust(&kustomization).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located<T> {
    value: T,
    path: PathId,
    parent_path: PathId,
}

impl<T> Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

fn load_kustomization(path: impl AsRef<Path>) -> anyhow::Result<Located<Kustomization>> {
    load_manifest(path)
}

fn load_component(path: impl AsRef<Path>) -> anyhow::Result<Located<Component>> {
    let component = load_manifest(path)?;
    // kind is required for to be explicitly specified for a `Component` to differentiate them from a `Kustomization`.
    // apiVersion is still optional.

    if component.type_meta.kind.is_none() {
        bail!(
            "Components referenced in `components` must have `kind: {}` specified",
            kind::Component
        );
    }

    Ok(component)
}

fn load_manifest<A, K>(path: impl AsRef<Path>) -> anyhow::Result<Located<Manifest<A, K>>>
where
    A: Symbol + serde::de::DeserializeOwned,
    K: Symbol + serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    if !path.exists() {
        return Err(anyhow::anyhow!("path does not exist: {}", path.pretty()));
    }

    let mut base = path
        .canonicalize()
        .with_context(|| format!("canonicalizing path {}", path.pretty()))?;

    if base.is_dir() {
        if base.join("kustomization.yaml").exists() && base.join("kustomization.yml").exists() {
            return Err(anyhow::anyhow!(
                "both kustomization.yaml and kustomization.yml exist in directory: {}",
                base.pretty()
            ));
        }

        if base.join("kustomization.yml").exists() {
            base.push("kustomization.yml");
        } else {
            base.push("kustomization.yaml");
        }
    }

    if base.file_name() != Some(OsStr::new("kustomization.yaml"))
        && base.file_name() != Some(OsStr::new("kustomization.yml"))
    {
        return Err(anyhow::anyhow!(
            "path is not a kustomization.yaml file: {}",
            base.pretty()
        ));
    }

    let file = std::fs::File::open(&base)
        .with_context(|| format!("loading manifest from path {}", base.pretty()))?;
    let mut manifest = yaml::from_reader::<Manifest<A, K>>(BufReader::new(file))?;

    let parent_path = PathId::make(base.parent().unwrap())?;

    // Change legacy patch fields into unified `patches` field
    let mut patches = mem::take(&mut manifest.patches).into_vec();
    for path_or_inline in mem::take(&mut manifest.patches_strategic_merge)
        .into_vec()
        .drain(..)
    {
        let path = parent_path.join(&path_or_inline);
        // This is actually what kustomize does to detect whether it's inline or a path. Unbelievable.
        if path.exists() {
            let resources =
                Resource::load_many(&path).context("loading strategic merge patches")?;
            patches.extend(resources.into_iter().map(|patch| Patch::StrategicMerge {
                patch,
                target: None,
            }));
        } else {
            let patch = yaml::from_str::<Resource>(&path_or_inline).with_context(|| {
                format!("parsing inline strategic merge patch at {}", base.display())
            })?;
            patches.push(Patch::StrategicMerge {
                patch,
                target: None,
            });
        }
    }

    patches.append(&mut mem::take(&mut manifest.patches_json6902).into_vec());
    manifest.patches = patches.into_boxed_slice();

    // Transform legacy `commonLabels` field into `labels`
    let mut labels = mem::take(&mut manifest.labels).into_vec();
    if !manifest.common_labels.is_empty() {
        labels.push(Label {
            pairs: mem::take(&mut manifest.common_labels),
            include_selectors: true,
            include_templates: false,
            fields: Default::default(),
        });
    }
    manifest.labels = labels.into_boxed_slice();

    if let Some(path) = manifest.openapi.as_ref().map(|api| &api.path) {
        openapi::v2::Spec::set_global_default_path(parent_path.join(path))?;
    }

    Ok(Located {
        value: manifest,
        parent_path,
        path: PathId::make(base)?,
    })
}

pub trait PathExt {
    fn pretty(&self) -> impl fmt::Display;
}

impl PathExt for Path {
    fn pretty(&self) -> impl fmt::Display {
        if let Ok(path) = self.strip_prefix(std::env::current_dir().unwrap_or_default()) {
            path.display()
        } else {
            self.display()
        }
    }
}
