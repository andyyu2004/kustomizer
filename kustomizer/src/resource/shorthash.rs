use std::borrow::Cow;
use std::collections::BTreeMap;

use anyhow::bail;

use crate::manifest::Str;

use super::Resource;

impl Resource {
    pub fn shorthash(&self) -> anyhow::Result<Str> {
        let encoded = match self.kind().as_str() {
            "ConfigMap" => encode_config_map(self)?,
            "Secret" => encode_secret(self)?,
            _ => bail!("Hash generation is only supported for kinds ConfigMap and Secret"),
        };

        // Match go's json.HTMLEscape behavior when marshalling json.
        // Sadly kustomize does not turn off this default behavior.
        let encoded = html_escape(&encoded);
        let hex = sha256::digest(encoded.as_ref());
        encode_hex(&hex)
    }
}

fn encode_config_map(resource: &Resource) -> anyhow::Result<String> {
    // Sort by key order similar to go's `json.Marshal`
    #[derive(serde::Serialize)]
    struct ConfigMap {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "binaryData")]
        binary_data: Option<json::Value>,
        data: json::Value,
        kind: &'static str,
        name: Str,
    }

    // Sort fields by key order
    let data = match resource.root().get("data") {
        Some(d) if d.as_object().is_some() => {
            let map: BTreeMap<String, json::Value> = d
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            json::to_value(map)?
        }
        _ => json::Value::String("".to_string()),
    };

    let binary_data = resource
        .root()
        .get("binaryData")
        .and_then(|d| d.as_object())
        .map(|obj| {
            let map: BTreeMap<String, json::Value> =
                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            json::to_value(map).unwrap()
        });

    let config_map = ConfigMap {
        binary_data,
        data,
        kind: "ConfigMap",
        // To get hashes to match kustomize, we ignore the name here?
        name: Default::default(),
        // name: resource.name().clone(),
    };

    Ok(json::to_string(&config_map)?)
}

fn encode_secret(resource: &Resource) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct Secret {
        data: json::Value,
        kind: &'static str,
        name: Str,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "stringData")]
        string_data: Option<json::Value>,
        #[serde(rename = "type")]
        secret_type: json::Value,
    }

    let data = match resource.root().get("data") {
        Some(d) if d.as_object().is_some() => {
            let map: BTreeMap<String, json::Value> = d
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            json::to_value(map)?
        }
        _ => json::Value::String("".to_string()),
    };

    let string_data = resource
        .root()
        .get("stringData")
        .and_then(|d| d.as_object())
        .map(|obj| {
            let map: BTreeMap<String, json::Value> =
                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            json::to_value(map).unwrap()
        });

    let secret_type = resource
        .root()
        .get("type")
        .cloned()
        .unwrap_or_else(|| json::Value::String("".to_string()));

    let secret = Secret {
        data,
        kind: "Secret",
        name: "".into(),
        string_data,
        secret_type,
    };

    Ok(json::to_string(&secret)?)
}

/// HTMLEscape escapes <, >, &, U+2028 and U+2029 characters in JSON strings
/// to make them safe for embedding in HTML <script> tags.
/// Ported from Go's encoding/json HTMLEscape function.
/// Returns a Cow to avoid allocation if no escaping is needed.
fn html_escape(src: &str) -> Cow<'_, str> {
    fn html_escape_slow(bytes: &[u8], first_escape: usize) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut dst = Vec::with_capacity(bytes.len());
        dst.extend_from_slice(&bytes[..first_escape]);

        let mut i = first_escape;
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'<' || c == b'>' || c == b'&' {
                dst.extend_from_slice(b"\\u00");
                dst.push(HEX[(c >> 4) as usize]);
                dst.push(HEX[(c & 0xF) as usize]);
                i += 1;
            } else if c == 0xE2
                && i + 2 < bytes.len()
                && bytes[i + 1] == 0x80
                && (bytes[i + 2] & !1) == 0xA8
            {
                dst.extend_from_slice(b"\\u202");
                dst.push(HEX[(bytes[i + 2] & 0xF) as usize]);
                i += 3;
            } else {
                dst.push(c);
                i += 1;
            }
        }

        String::from_utf8(dst).expect("html_escape produced valid UTF-8")
    }

    let bytes = src.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'<' || c == b'>' || c == b'&' {
            // Found a character that needs escaping, do the full pass
            return Cow::Owned(html_escape_slow(bytes, i));
        }
        if c == 0xE2 && i + 2 < bytes.len() && bytes[i + 1] == 0x80 && (bytes[i + 2] & !1) == 0xA8 {
            // Found U+2028 or U+2029 that needs escaping
            return Cow::Owned(html_escape_slow(bytes, i));
        }
        i += 1;
    }

    // no escaping needed, return a borrowed reference
    Cow::Borrowed(src)
}

// Copied from https://github.com/kubernetes/kubernetes
// /blob/master/pkg/kubectl/util/hash/hash.go
fn encode_hex(hex: &str) -> anyhow::Result<Str> {
    if hex.len() < 10 {
        bail!("input hex string must be at least 10 characters long");
    }

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

    assert_eq!(out.len(), 10);
    Ok(out)
}

#[cfg(test)]
mod tests;
