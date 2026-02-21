use anyhow::Context;

use crate::{
    manifest::{self, GeneratorOptions},
    resource::{Annotations, Gvk, Metadata, Object, ResId, Resource},
};

use super::{
    common::{DataEncoding, merge_options, name_generated_resource, process_key_value_sources},
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

impl Generator for SecretGenerator<'_> {
    #[tracing::instrument(skip_all, name = "generate_secret", fields(workdir = %workdir.display()))]
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

        let (data, empty) = process_key_value_sources(
            workdir,
            &generator.sources,
            DataEncoding::Secret,
            "SecretGenerator",
        )
        .await?;
        assert!(
            empty.is_empty(),
            "this is part of the contract to be empty when encoding is Secret"
        );

        let mut root = Object::from_iter([("data".into(), json::Value::Object(data))]);

        if immutable {
            root.insert("immutable".into(), json::Value::Bool(true));
        }

        root.insert("type".into(), json::Value::String(generator.ty.to_string()));

        // Convert empty namespace to None for resource output
        let namespace = generator.namespace.as_ref().filter(|s| !s.is_empty()).cloned();

        let secret = Resource::new(
            ResId {
                gvk: Gvk {
                    group: "".into(),
                    version: "v1".into(),
                    kind: "Secret".into(),
                },
                name: Default::default(),
                namespace: namespace.clone(),
            },
            Metadata {
                name: Default::default(),
                namespace,
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

        name_generated_resource(secret, generator.name.clone(), disable_name_suffix_hash)
    }
}
