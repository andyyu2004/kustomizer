use std::collections::BTreeMap;

use anyhow::bail;

use crate::manifest::Str;

use super::Resource;

impl Resource {
    pub fn shorthash(&self) -> anyhow::Result<Str> {
        let encoded = match self.kind().as_str() {
            "ConfigMap" => {
                #[derive(serde::Serialize)]
                struct ConfigMap {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = "binaryData")]
                    binary_data: Option<serde_json::Value>,
                    data: serde_json::Value,
                    kind: &'static str,
                    name: Str,
                }

                let data = match self.root().get("data") {
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

                let binary_data = self
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
                    name: "".into(),
                };

                serde_json::to_string(&config_map)?
            }
            "Secret" => {
                bail!("Implement hash for Secret");
            }
            _ => {
                bail!("Implement hash for other resource types");
            }
        };

        let hex = sha256::digest(encoded);
        Ok(encode_hex(&hex))
    }
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
mod tests {
    use crate::resource::Resource;

    fn create_resource_from_yaml(yaml: &str) -> anyhow::Result<Resource> {
        let value: serde_json::Value = serde_yaml::from_str(yaml)?;
        let resource: Resource = serde_json::from_value(value)?;
        Ok(resource)
    }

    fn skip_rest(desc: &str, result: &anyhow::Result<String>, expected_err: &str) -> bool {
        match result {
            Err(err) => {
                if expected_err.is_empty() {
                    panic!("case '{}', expect nil error but got '{}'", desc, err);
                } else if !err.to_string().contains(expected_err) {
                    panic!(
                        "case '{}', expect error to contain '{}' but got '{}'",
                        desc, expected_err, err
                    );
                }
                true
            }
            Ok(_) => {
                if !expected_err.is_empty() {
                    panic!(
                        "case '{}', expect error to contain '{}' but got nil error",
                        desc, expected_err
                    );
                }
                false
            }
        }
    }

    #[test]
    fn test_config_map_hash() -> anyhow::Result<()> {
        let cases = vec![
            // empty map
            (
                "empty data",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: """#,
                "6ct58987ht",
                "",
            ),
            // one key
            (
                "one key",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
data:
  one: """#,
                "9g67k2htb6",
                "",
            ),
            // three keys (tests sorting order)
            (
                "three keys",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
data:
  two: 2
  one: ""
  three: 3"#,
                "7757f9kkct",
                "",
            ),
            // empty binary data map
            (
                "empty binary data",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: """#,
                "6ct58987ht",
                "",
            ),
            // one key with binary data
            (
                "one key with binary data",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
binaryData:
  one: """#,
                "6mtk2m274t",
                "",
            ),
            // three keys with binary data (tests sorting order)
            (
                "three keys with binary data",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
binaryData:
  two: 2
  one: ""
  three: 3"#,
                "9th7kc28dg",
                "",
            ),
            // two keys, one with string and another with binary data
            (
                "two keys with one each",
                r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
data:
  one: ""
binaryData:
  two: """#,
                "698h7c7t9m",
                "",
            ),
        ];

        for (desc, cm_yaml, expected_hash, err_msg) in cases {
            let resource = create_resource_from_yaml(cm_yaml)?;
            let result = resource.shorthash().map(|s| s.to_string());

            if skip_rest(desc, &result, err_msg) {
                continue;
            }

            let hashed = result?;
            assert_eq!(
                expected_hash, hashed,
                "case '{}', expect hash '{}' but got '{}'",
                desc, expected_hash, hashed
            );
        }
        Ok(())
    }
}
