mod build;
mod intern;
pub mod manifest;
mod resmap;
mod resource;

use std::{io::Write, ops::Deref, path::Path};

use anyhow::Context;

pub use self::intern::PathId;

use self::manifest::{Component, Kustomization, Manifest, Symbol};

pub fn build(path: impl AsRef<Path>, out: &mut dyn Write) -> anyhow::Result<()> {
    let kustomization = load_kustomization(path)?;
    build::Builder::default().build(&kustomization, out)
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

    let mut path = path
        .canonicalize()
        .with_context(|| format!("canonicalizing path {}", path.display()))?;
    if path.is_dir() {
        path.push("kustomization.yaml");
    }

    let file = std::fs::File::open(&path)
        .with_context(|| format!("loading manifest from path {}", path.display()))?;
    let value = serde_yaml::from_reader(file)?;
    Ok(Located {
        value,
        parent_path: PathId::make(path.parent().unwrap())?,
        path: PathId::make(path)?,
    })
}

fn load_file(path: impl AsRef<Path>) -> anyhow::Result<String> {
    std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("loading file from path {}", path.as_ref().display()))
}

fn load_yaml<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let id = PathId::make(path)
        .with_context(|| format!("loading resource from path {}", path.display()))?;
    let file = std::fs::File::open(id)?;
    Ok(serde_yaml::from_reader(file)?)
}

#[cfg(test)]
#[test]
fn hack_test_tmp() -> anyhow::Result<()> {
    let status = std::process::Command::new("bash").arg("test.sh").status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("test.sh failed with status: {}", status));
    }
    Ok(())
}
