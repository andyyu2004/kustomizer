use std::collections::HashMap;

use crate::{
    Located, PathId,
    manifest::{Component, Kustomization, Manifest},
};
use anyhow::Context;

#[derive(Debug, Default)]
pub struct Builder {
    components: HashMap<PathId, Component>,
    resource: HashMap<PathId, serde_yaml::Value>,
}

impl Builder {
    pub async fn build(mut self, kustomization: &Located<Kustomization>) -> anyhow::Result<()> {
        self.gather(kustomization).await?;
        dbg!(&self);
        Ok(())
    }

    async fn gather(&mut self, kustomization: &Located<Kustomization>) -> anyhow::Result<()> {
        self.traverse(kustomization)?;

        Ok(())
    }

    fn traverse<A, K>(&mut self, manifest: &Located<Manifest<A, K>>) -> anyhow::Result<()> {
        let base_path = manifest
            .path
            .parent()
            .expect("this is a file so it has a parent")
            .canonicalize()?;

        for path in &manifest.resources {
            let path = PathId::make(base_path.join(path))
                .with_context(|| format!("canonicalizing resource path {}", path.display()))?;

            if self.resource.contains_key(&path) {
                continue;
            }

            let resource = crate::load_resource::<serde_yaml::Value>(path)
                .with_context(|| format!("loading resource {}", path.display()))?;
            assert!(self.resource.insert(path, resource).is_none());
        }

        for path in &manifest.components {
            let path = PathId::make(base_path.join(path))
                .with_context(|| format!("canonicalizing component path {}", path.display()))?;

            if self.components.contains_key(&path) {
                continue;
            }

            let component = crate::load_component(path)
                .with_context(|| format!("loading component {}", path.display()))?;

            // Insert a placeholder to avoid cycles causing overflow. TODO detect cycles and report them.
            assert!(self.components.insert(path, Component::default()).is_none());
            self.traverse(&component)?;
            assert_eq!(
                self.components.insert(path, component.value),
                Some(Component::default())
            );
        }

        Ok(())
    }
}
