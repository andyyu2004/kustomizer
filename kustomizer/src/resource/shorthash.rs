use std::collections::BTreeMap;

use anyhow::{Context as _, bail};

use crate::manifest::Str;

use super::Resource;

impl Resource {
    pub fn shorthash(&self) -> anyhow::Result<Str> {
        let encoded = match self.kind().as_str() {
            "ConfigMap" => {
                #[derive(serde::Serialize)]
                struct ConfigMap {
                    kind: &'static str,
                    name: Str,
                    data: BTreeMap<Str, Box<str>>,
                }

                let Some(data) = self.root().get("data") else {
                    bail!("ConfigMap missing 'data' field");
                };

                let data = data
                    .as_object()
                    .context("ConfigMap 'data' field is not an object")?
                    .iter()
                    .map(|(k, v)| {
                        let v = v
                            .as_str()
                            .context("ConfigMap 'data' value is not a string")?;
                        Ok((k.into(), v.into()))
                    })
                    .collect::<anyhow::Result<_>>()?;

                let config_map = ConfigMap {
                    kind: "ConfigMap",
                    name: self.name().clone(),
                    data,
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

// func encode(hex string) (string, error) {
// 	if len(hex) < 10 {
// 		return "", fmt.Errorf(
// 			"input length must be at least 10")
// 	}
// 	enc := []rune(hex[:10])
// 	for i := range enc {
// 		switch enc[i] {
// 		case '0':
// 			enc[i] = 'g'
// 		case '1':
// 			enc[i] = 'h'
// 		case '3':
// 			enc[i] = 'k'
// 		case 'a':
// 			enc[i] = 'm'
// 		case 'e':
// 			enc[i] = 't'
// 		}
// 	}
// 	return string(enc), nil
// }
