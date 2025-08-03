use anyhow::{Context, bail};

use crate::{
    PathExt, manifest,
    resource::{Annotations, Gvk, Metadata, ResId, Resource},
};

use super::*;

pub struct ConfigMapGenerator<'a>(&'a [manifest::Generator]);

impl<'a> ConfigMapGenerator<'a> {
    pub fn new(generators: &'a [manifest::Generator]) -> Self {
        Self(generators)
    }
}

#[async_trait::async_trait]
impl Generator for ConfigMapGenerator<'_> {
    async fn generate(
        &mut self,
        workdir: &Path,
        _input: &ResourceList,
    ) -> anyhow::Result<ResourceList> {
        let mut resources = Vec::with_capacity(self.0.len());

        for generator in self.0 {
            resources.push(
                generate(workdir, generator)
                    .await
                    .with_context(|| format!("failed to generate ConfigMap {}", generator.name))?,
            );
        }

        Ok(ResourceList::new(resources))
    }
}

async fn generate(wd: &Path, generator: &manifest::Generator) -> anyhow::Result<Resource> {
    if !generator.sources.literals.is_empty() {
        bail!("ConfigMapGenerator does not support literal sources");
    }

    let mut mapping = serde_yaml::Mapping::new();

    for kv in &generator.sources.files {
        let path = wd.join(&kv.value);
        let key = kv.key.clone().unwrap_or_else(|| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into()
        });
        let data = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read file {}", path.pretty()))?;

        if mapping
            .insert(
                serde_yaml::Value::String(key.to_string()),
                serde_yaml::Value::String(data),
            )
            .is_some()
        {
            bail!("duplicate key `{key}` in ConfigMapGenerator sources");
        }
    }

    let root = serde_yaml::Mapping::from_iter([(
        serde_yaml::Value::String("data".into()),
        serde_yaml::Value::Mapping(mapping),
    )]);

    let configmap = Resource {
        id: ResId {
            gvk: Gvk {
                group: "".into(),
                version: "v1".into(),
                kind: "ConfigMap".into(),
            },
            name: generator.name.clone(),
            namespace: generator.namespace.clone(),
        },
        metadata: Metadata {
            name: generator.name.clone(),
            namespace: generator.namespace.clone(),
            annotations: Annotations {
                behavior: generator.behavior,
                ..Default::default()
            },
            ..Default::default()
        },
        root: serde_yaml::Value::Mapping(root),
    };

    Ok(configmap)
}
