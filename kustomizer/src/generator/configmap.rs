use anyhow::{Context, bail};

use crate::{
    PathExt,
    manifest::{self, GeneratorOptions},
    resource::{Annotations, AnyObject, Gvk, Metadata, ResId, Resource},
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
                    .with_context(|| {
                        format!("failed to generate ConfigMap `{}`", generator.name)
                    })?,
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
            bail!("ConfigMapGenerator does not support literal sources yet");
        }

        let GeneratorOptions {
            labels,
            annotations,
            disable_name_suffix_hash,
            immutable,
        } = merge_options(self.options, &generator.options);

        let mut object = AnyObject::new();

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

            if object
                .insert(key.to_string(), serde_json::Value::String(data))
                .is_some()
            {
                bail!("duplicate key `{key}` in ConfigMapGenerator sources");
            }
        }

        let mut root = AnyObject::from_iter([("data".into(), serde_json::Value::Object(object))]);

        if immutable {
            root.insert("immutable".into(), serde_json::Value::Bool(true));
        }

        let config_map = Resource::new(
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
                    behavior: Some(generator.behavior),
                    rest: annotations,
                    ..Default::default()
                },
                labels: labels.clone(),
                ..Default::default()
            },
            root,
        )?;

        let suffix_hash = disable_name_suffix_hash.map(|v| !v).unwrap_or(true);

        if suffix_hash {
            // TODO need to make sure all references to this ConfigMap are updated too
            Ok(config_map.with_name_suffix_hash()?)
        } else {
            Ok(config_map)
        }
    }
}
