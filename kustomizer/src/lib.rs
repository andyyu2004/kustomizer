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
mod serde_ex;
mod transform;

use core::fmt;
use std::{ffi::OsStr, ops::Deref, path::Path};

use anyhow::Context;

pub use self::intern::PathId;

use self::{
    manifest::{Component, Kustomization, Manifest, Symbol},
    resmap::ResourceMap,
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
    load_manifest(path)
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

    let mut path = path
        .canonicalize()
        .with_context(|| format!("canonicalizing path {}", path.pretty()))?;
    if path.is_dir() {
        path.push("kustomization.yaml");
    }

    if path.file_name() != Some(OsStr::new("kustomization.yaml")) {
        return Err(anyhow::anyhow!(
            "path is not a kustomization.yaml file: {}",
            path.pretty()
        ));
    }

    let file = std::fs::File::open(&path)
        .with_context(|| format!("loading manifest from path {}", path.pretty()))?;
    let value = serde_yaml::from_reader(file)?;
    Ok(Located {
        value,
        parent_path: PathId::make(path.parent().unwrap())?,
        path: PathId::make(path)?,
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

#[cfg(test)]
mod hack {
    use std::process::{Command, Stdio};

    #[test]
    #[ignore]
    fn test_partly_dev9() -> anyhow::Result<()> {
        let output = Command::new("bash")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("test-3.sh")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "test.sh failed with status {}: {}\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ));
        }
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_tmp_1() -> anyhow::Result<()> {
        let output = Command::new("bash")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("test-1.sh")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "test-1.sh failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
            ));
        }
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_tmp_2() -> anyhow::Result<()> {
        let output = Command::new("bash")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("test-2.sh")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "test-2.sh failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
            ));
        }
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_tmp_3() -> anyhow::Result<()> {
        let output = Command::new("bash")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("test-3.sh")
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "test-3.sh failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
            ));
        }
        Ok(())
    }
}
