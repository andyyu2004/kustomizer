use std::collections::BTreeMap;

use anyhow::bail;

use crate::manifest::Str;

use super::Resource;

impl Resource {
    pub fn shorthash(&self) -> anyhow::Result<Str> {
        let encoded = match self.kind().as_str() {
            "ConfigMap" => encode_config_map(self)?,
            "Secret" => encode_secret(self)?,
            _ => {
                bail!("Implement hash for other resource types");
            }
        };

        let hex = sha256::digest(encoded);
        Ok(encode_hex(&hex))
    }
}

fn encode_config_map(resource: &Resource) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct ConfigMap {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "binaryData")]
        binary_data: Option<serde_json::Value>,
        data: serde_json::Value,
        kind: &'static str,
        name: Str,
    }

    let data = match resource.root().get("data") {
        Some(d) if d.as_object().is_some() => {
            let map: BTreeMap<String, serde_json::Value> = d
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            serde_json::to_value(map)?
        }
        _ => serde_json::Value::String("".to_string()),
    };

    let binary_data = resource
        .root()
        .get("binaryData")
        .and_then(|d| d.as_object())
        .map(|obj| {
            let map: BTreeMap<String, serde_json::Value> =
                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            serde_json::to_value(map).unwrap()
        });

    let config_map = ConfigMap {
        binary_data,
        data,
        kind: "ConfigMap",
        name: resource.name().clone(),
    };

    Ok(serde_json::to_string(&config_map)?)
}

fn encode_secret(resource: &Resource) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct Secret {
        data: serde_json::Value,
        kind: &'static str,
        name: Str,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "stringData")]
        string_data: Option<serde_json::Value>,
        #[serde(rename = "type")]
        secret_type: serde_json::Value,
    }

    let data = match resource.root().get("data") {
        Some(d) if d.as_object().is_some() => {
            let map: BTreeMap<String, serde_json::Value> = d
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            serde_json::to_value(map)?
        }
        _ => serde_json::Value::String("".to_string()),
    };

    let string_data = resource
        .root()
        .get("stringData")
        .and_then(|d| d.as_object())
        .map(|obj| {
            let map: BTreeMap<String, serde_json::Value> =
                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            serde_json::to_value(map).unwrap()
        });

    let secret_type = resource
        .root()
        .get("type")
        .cloned()
        .unwrap_or_else(|| serde_json::Value::String("".to_string()));

    let secret = Secret {
        data,
        kind: "Secret",
        name: "".into(),
        string_data,
        secret_type,
    };

    Ok(serde_json::to_string(&secret)?)
}

// Copied from https://github.com/kubernetes/kubernetes
// /blob/master/pkg/kubectl/util/hash/hash.go
fn encode_hex(hex: &str) -> Str {
    let mut out = Str::with_capacity(10);
    for c in hex.chars().take(10) {
        let c = match c {
            '0' => 'g',
            '1' => 'h',
            '3' => 'k',
            'a' => 'm',
            'e' => 't',
            _ => c,
        };
        out.push(c);
    }

    out
}

#[cfg(test)]
mod tests;
