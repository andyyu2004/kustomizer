use serde::Deserialize;

use crate::manifest::{Gvk, ResId, Str};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    id: ResId,
    manifest: serde_yaml::Value,
}

impl<'de> Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Res {
            api_version: Str,
            kind: Str,
            metadata: Meta,
            #[serde(flatten)]
            manifest: serde_yaml::Value,
        }

        #[derive(Debug, Deserialize)]
        struct Meta {
            name: Str,
            namespace: Option<Str>,
        }

        let partial = Res::deserialize(deserializer)?;

        let (group, version) = partial
            .api_version
            .split_once('/')
            .map_or(("".into(), partial.api_version.clone()), |(g, v)| {
                (g.into(), v.into())
            });

        let id = ResId {
            gvk: Gvk {
                group,
                version,
                kind: partial.kind,
            },
            name: partial.metadata.name,
            namespace: partial.metadata.namespace,
        };

        Ok(Resource {
            id,
            manifest: partial.manifest,
        })
    }
}
