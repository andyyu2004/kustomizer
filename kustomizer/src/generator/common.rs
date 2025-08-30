use anyhow::{Context, bail};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use tokio::io::AsyncBufReadExt as _;

use crate::{
    PathExt,
    manifest::{GeneratorOptions, KeyValuePairSources, Str},
    resource::{Object, Resource},
};

use std::path::Path;

pub fn merge_options(global: &GeneratorOptions, local: &GeneratorOptions) -> GeneratorOptions {
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

pub enum DataEncoding {
    Raw,
    Base64,
}

pub async fn process_key_value_sources(
    workdir: &Path,
    sources: &KeyValuePairSources,
    encoding: DataEncoding,
    resource_type: &str,
) -> anyhow::Result<Object> {
    let mut object = Object::new();

    for kv in &sources.literals {
        let value = match encoding {
            DataEncoding::Raw => kv.value.to_string(),
            DataEncoding::Base64 => base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                kv.value.as_bytes(),
            ),
        };

        if object
            .insert(kv.key.to_string(), serde_json::Value::String(value))
            .is_some()
        {
            bail!("duplicate key `{}` in {resource_type} sources", kv.key);
        }
    }

    for kv in &sources.files {
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

        let encoded_data = match encoding {
            DataEncoding::Raw => data,
            DataEncoding::Base64 => BASE64_STANDARD.encode(data.as_bytes()),
        };

        if object
            .insert(key.to_string(), serde_json::Value::String(encoded_data))
            .is_some()
        {
            bail!("duplicate key `{key}` in {resource_type} sources");
        }
    }

    for path in &sources.envs {
        let path = workdir.join(path);
        let file = tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("failed to read env file {}", path.pretty()))?;
        let mut lines = tokio::io::BufReader::new(file).lines();
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                bail!("invalid line in env file {}: {line}", path.pretty());
            };

            let value = match encoding {
                DataEncoding::Raw => value.to_string(),
                DataEncoding::Base64 => BASE64_STANDARD.encode(value.as_bytes()),
            };

            if object
                .insert(key.to_string(), serde_json::Value::String(value))
                .is_some()
            {
                bail!("duplicate key `{key}` in {} sources", resource_type);
            }
        }
    }

    Ok(object)
}

pub fn name_generated_resource(
    resource: Resource,
    name: Str,
    disable_name_suffix_hash: Option<bool>,
) -> anyhow::Result<Resource> {
    let mut resource = resource.with_name(name);
    let suffix_hash = disable_name_suffix_hash.map(|v| !v).unwrap_or(true);
    if suffix_hash {
        resource
            .metadata_mut()
            .make_annotations_mut()
            .set_needs_hash();
    }
    Ok(resource)
}
