use anyhow::Context;

use crate::{
    manifest::{self, GeneratorOptions},
    resource::{Annotations, Gvk, Metadata, Object, ResId, Resource},
};

use super::{
    common::{DataEncoding, merge_options, name_generated_resource, process_key_value_sources},
    *,
};

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

impl ConfigMapGenerator<'_> {
    async fn generate_one(
        &self,
        workdir: &Path,
        generator: &manifest::Generator,
    ) -> anyhow::Result<Resource> {
        let GeneratorOptions {
            labels,
            annotations,
            disable_name_suffix_hash,
            immutable,
        } = merge_options(self.options, &generator.options);

        let object = process_key_value_sources(
            workdir,
            &generator.sources,
            DataEncoding::Raw,
            "ConfigMapGenerator",
        )
        .await?;

        let mut root = Object::from_iter([("data".into(), serde_json::Value::Object(object))]);

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
                name: Default::default(),
                namespace: generator.namespace.clone(),
            },
            Metadata {
                name: Default::default(),
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

        name_generated_resource(config_map, &generator.name, disable_name_suffix_hash)
    }
}
