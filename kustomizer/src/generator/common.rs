use anyhow::{Context, bail};
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

pub(crate) enum DataEncoding {
    ConfigMap,
    Secret,
}

// Strips surrounding single or double quotes from a string, if present.
fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() < 2 || bytes[0] != bytes[bytes.len() - 1] {
        return s;
    }

    if bytes[0] == b'\'' || bytes[0] == b'"' {
        return &s[1..bytes.len() - 1];
    }

    s
}

/// Processes key-value pair sources and returns (`data`, `binary_data`) objects.
/// `data` contains most of the data, `binary_data` contains non-utf8 values only when `encoding == DataEncoding::ConfigMap`.
pub(crate) async fn process_key_value_sources(
    workdir: &Path,
    sources: &KeyValuePairSources,
    encoding: DataEncoding,
    resource_type: &str,
) -> anyhow::Result<(Object, Object)> {
    let mut data = Object::new();
    let mut binary_data = Object::new();

    for kv in &sources.literals {
        let value = strip_quotes(&kv.value);
        let value = match encoding {
            DataEncoding::ConfigMap => value.to_string(),
            DataEncoding::Secret => base64_encode(value.as_bytes()),
        };

        if data
            .insert(kv.key.to_string(), json::Value::String(value))
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
        let value = tokio::fs::read(&path).await.with_context(|| {
            format!("failed to read file as key value source {}", path.pretty())
        })?;

        match encoding {
            DataEncoding::ConfigMap => match String::from_utf8(value) {
                Ok(s) => {
                    if binary_data.contains_key(key.as_str())
                        || data
                            .insert(key.to_string(), json::Value::String(s))
                            .is_some()
                    {
                        bail!("duplicate key `{key}` in {resource_type} sources");
                    }
                }
                Err(err) => {
                    let value = base64_encode(err.as_bytes());
                    if data.contains_key(key.as_str())
                        || binary_data
                            .insert(key.to_string(), json::Value::String(value))
                            .is_some()
                    {
                        bail!("duplicate key `{key}` in {resource_type} sources");
                    }
                }
            },
            DataEncoding::Secret => {
                if data
                    .insert(key.to_string(), json::Value::String(base64_encode(&value)))
                    .is_some()
                {
                    bail!("duplicate key `{key}` in {resource_type} sources");
                }
            }
        };
    }

    for path in &sources.envs {
        let path = workdir.join(path);
        let file = tokio::fs::File::open(&path)
            .await
            .with_context(|| format!("failed to read env file {}", path.pretty()))?;
        let mut lines = tokio::io::BufReader::new(file).lines();
        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            // Split on the first '=' character.
            // If there is no '=', the value is the empty string.
            let (key, value) = line.split_once('=').unwrap_or((line, ""));

            let value = match encoding {
                DataEncoding::ConfigMap => value.to_string(),
                DataEncoding::Secret => base64_encode(value.as_bytes()),
            };

            if data
                .insert(key.to_string(), json::Value::String(value))
                .is_some()
            {
                bail!("duplicate key `{key}` in {resource_type} sources")
            }
        }
    }

    Ok((data, binary_data))
}

/// Implement matching base64 encoding to kustomize.
/// 70 characters per line with padding.
fn base64_encode(s: &[u8]) -> String {
    use base64::{Engine, prelude::BASE64_STANDARD};
    const LINE_LEN: usize = 70;

    let encoded = BASE64_STANDARD.encode(s);
    let enc_len = encoded.len();
    let lines = enc_len / LINE_LEN + 1;

    if lines <= 1 {
        return encoded;
    }

    let mut result = String::with_capacity(enc_len + lines);
    for chunk in encoded.as_bytes().chunks(LINE_LEN) {
        result.push_str(std::str::from_utf8(chunk).unwrap());
        result.push('\n');
    }

    result
}

#[cfg(test)]
#[test]
fn test_base64_encode() -> anyhow::Result<()> {
    use base64::Engine as _;
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    let mut reader = BufReader::new(File::open("/dev/urandom")?);

    for i in 0..1000 {
        let mut buf = vec![0u8; i];
        reader.read_exact(&mut buf)?;

        let encoded = base64_encode(&buf);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded.replace('\n', ""))
            .unwrap();
        assert_eq!(buf, decoded, "mismatch for length {i}");
    }

    Ok(())
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
            .make_metadata_mut()
            .make_annotations_mut()
            .set_needs_hash();
    }
    Ok(resource)
}
