use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use self::manifest::{Component, Kustomization, Manifest, Symbol};

pub mod build;
pub mod manifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located<T> {
    pub value: T,
    pub path: PathBuf,
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
    Ok(Located { value, path })
}
