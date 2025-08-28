use anyhow::{Context, bail};

use crate::{
    PathExt,
    manifest::{GeneratorOptions, KeyValuePairSources},
    resource::Object,
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
            DataEncoding::Base64 => {
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, kv.value.as_bytes())
            }
        };

        if object
            .insert(
                kv.key.to_string(),
                serde_json::Value::String(value),
            )
            .is_some()
        {
            bail!("duplicate key `{}` in {} sources", kv.key, resource_type);
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
            DataEncoding::Base64 => {
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data.as_bytes())
            }
        };

        if object
            .insert(key.to_string(), serde_json::Value::String(encoded_data))
            .is_some()
        {
            bail!("duplicate key `{key}` in {} sources", resource_type);
        }
    }

    Ok(object)
}

pub fn apply_hash_suffix_if_needed(
    resource: crate::resource::Resource,
    disable_name_suffix_hash: Option<bool>,
) -> anyhow::Result<crate::resource::Resource> {
    let suffix_hash = disable_name_suffix_hash.map(|v| !v).unwrap_or(true);

    if suffix_hash {
        resource.with_name_suffix_hash()
    } else {
        Ok(resource)
    }
}