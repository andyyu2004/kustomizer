use anyhow::Context;

use crate::{
    manifest::{self, GeneratorOptions},
    resource::{Annotations, Gvk, Metadata, Object, ResId, Resource},
};

use super::{
    common::{DataEncoding, apply_hash_suffix_if_needed, merge_options, process_key_value_sources},
    *,
};

pub struct SecretGenerator<'a> {
    generators: &'a [manifest::SecretGenerator],
    options: &'a GeneratorOptions,
}

impl<'a> SecretGenerator<'a> {
    pub fn new(generators: &'a [manifest::SecretGenerator], options: &'a GeneratorOptions) -> Self {
        Self {
            generators,
            options,
        }
    }
}

#[async_trait::async_trait]
impl Generator for SecretGenerator<'_> {
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
                    .with_context(|| format!("failed to generate Secret `{}`", generator.name))?,
            );
        }

        Ok(ResourceList::new(resources))
    }
}

impl SecretGenerator<'_> {
    async fn generate_one(
        &self,
        workdir: &Path,
        generator: &manifest::SecretGenerator,
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
            DataEncoding::Base64,
            "SecretGenerator",
        ).await?;

        let mut root = Object::from_iter([("data".into(), serde_json::Value::Object(object))]);

        if immutable {
            root.insert("immutable".into(), serde_json::Value::Bool(true));
        }

        let secret_type = match generator.ty {
            manifest::SecretType::Opaque => "Opaque",
        };
        root.insert(
            "type".into(),
            serde_json::Value::String(secret_type.to_string()),
        );

        let secret = Resource::new(
            ResId {
                gvk: Gvk {
                    group: "".into(),
                    version: "v1".into(),
                    kind: "Secret".into(),
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

        apply_hash_suffix_if_needed(secret, disable_name_suffix_hash)
    }
}

