use anyhow::{Context, bail};

use crate::{
    PathExt,
    manifest::{self, GeneratorOptions},
    resource::{Annotations, Gvk, Metadata, ResId, Resource},
};

use super::*;

pub struct ConfigMapGenerator<'a> {
    generators: &'a [manifest::Generator],
    options: &'a GeneratorOptions,
}

impl<'a> ConfigMapGenerator<'a> {
    pub fn new(generators: &'a [manifest::Generator], options: &'a GeneratorOptions) -> Self {
        Self {
            generators,
            options,
        }
    }
}

#[async_trait::async_trait]
impl Generator for ConfigMapGenerator<'_> {
    async fn generate(
        &mut self,
        workdir: &Path,
        _input: &ResourceList,
    ) -> anyhow::Result<ResourceList> {
        let mut resources = Vec::with_capacity(self.generators.len());

        for generator in self.generators {
            resources.push(
                self.generate_one(workdir, generator)
                    .await
                    .with_context(|| format!("failed to generate ConfigMap {}", generator.name))?,
            );
        }

        Ok(ResourceList::new(resources))
    }
}

fn merge_options(global: &GeneratorOptions, local: &GeneratorOptions) -> GeneratorOptions {
    GeneratorOptions {
        labels: global
            .labels
            .iter()
            .chain(&local.labels)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        annotations: global
            .annotations
            .iter()
            .chain(&local.annotations)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        disable_name_suffix_hash: local
            .disable_name_suffix_hash
            .or(global.disable_name_suffix_hash),
        immutable: global.immutable || local.immutable,
    }
}

impl ConfigMapGenerator<'_> {
    async fn generate_one(
        &self,
        workdir: &Path,
        generator: &manifest::Generator,
    ) -> anyhow::Result<Resource> {
        if !generator.sources.literals.is_empty() {
            bail!("ConfigMapGenerator does not support literal sources");
        }

        let GeneratorOptions {
            labels,
            annotations,
            disable_name_suffix_hash,
            immutable,
        } = merge_options(self.options, &generator.options);

        if disable_name_suffix_hash.unwrap_or(false) {
            bail!("ConfigMapGenerator does not support enabling name suffix hash yet");
        }

        let mut mapping = serde_yaml::Mapping::new();

        for kv in &generator.sources.files {
            let path = workdir.join(&kv.value);
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

        let mut root = serde_yaml::Mapping::from_iter([(
            serde_yaml::Value::String("data".into()),
            serde_yaml::Value::Mapping(mapping),
        )]);

        if immutable {
            root.insert(
                serde_yaml::Value::String("immutable".into()),
                serde_yaml::Value::Bool(true),
            );
        }

        Resource::new(
            ResId {
                gvk: Gvk {
                    group: "".into(),
                    version: "v1".into(),
                    kind: "ConfigMap".into(),
                },
                name: generator.name.clone(),
                namespace: generator.namespace.clone(),
            },
            Metadata {
                name: generator.name.clone(),
                namespace: generator.namespace.clone(),
                annotations: Annotations {
                    behavior: generator.behavior,
                    rest: annotations,
                    ..Default::default()
                },
                labels: labels.clone(),
                ..Default::default()
            },
            root,
        )
    }
}
