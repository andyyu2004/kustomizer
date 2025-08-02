pub mod build;
mod intern;
pub mod manifest;

use std::{ops::Deref, path::Path};

use anyhow::Context;

pub use self::intern::PathId;

use self::manifest::{Component, Kustomization, Manifest, Symbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located<T> {
    pub value: T,
    pub path: PathId,
}

impl<T> Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub fn load_kustomization(path: impl AsRef<Path>) -> anyhow::Result<Located<Kustomization>> {
    load_manifest(path)
}

pub fn load_component(path: impl AsRef<Path>) -> anyhow::Result<Located<Component>> {
    load_manifest(path)
}

fn load_manifest<A, K>(path: impl AsRef<Path>) -> anyhow::Result<Located<Manifest<A, K>>>
where
    A: Symbol + serde::de::DeserializeOwned,
    K: Symbol + serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "load manifest: path does not exist: {}",
            path.display()
        ));
    }

    let mut path = path.canonicalize()?;
    if path.is_dir() {
        path.push("kustomization.yaml");
    }

    let file = std::fs::File::open(&path)?;
    let value = serde_yaml::from_reader(file)?;
    Ok(Located {
        value,
        path: PathId::make(path)?,
    })
}

fn load_file(path: impl AsRef<Path>) -> anyhow::Result<String> {
    std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("loading file from path {}", path.as_ref().display()))
}

fn load_resource<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let id = PathId::make(path)
        .with_context(|| format!("loading resource from path {}", path.display()))?;
    let file = std::fs::File::open(id)?;
    Ok(serde_yaml::from_reader(file)?)
}
