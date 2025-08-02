use std::fmt;

use compact_str::format_compact;
use serde::{Deserialize, Serialize};

use crate::manifest::Str;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Gvk {
    pub group: Str,
    pub version: Str,
    pub kind: Str,
}

impl fmt::Display for Gvk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.group.is_empty() {
            write!(f, "{}/{}", self.version, self.kind)
        } else {
            write!(f, "{}/{}/{}", self.group, self.version, self.kind)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResId {
    #[serde(flatten)]
    pub gvk: Gvk,
    pub name: Str,
    pub namespace: Option<Str>,
}

impl fmt::Display for ResId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}/{}/{}", namespace, self.gvk.kind, self.name)?;
        } else {
            write!(f, "{}/{}", self.gvk.kind, self.name)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: ResId,
    pub metadata: serde_yaml::Value,
    pub manifest: serde_yaml::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Res {
    api_version: Str,
    kind: Str,
    metadata: Metadata,
    #[serde(flatten)]
    manifest: serde_yaml::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Metadata {
    name: Str,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<Str>,
    #[serde(flatten)]
    rest: serde_yaml::Value,
}

impl Serialize for Resource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let api_version = if self.id.gvk.group.is_empty() {
            self.id.gvk.version.clone()
        } else {
            format_compact!("{}/{}", self.id.gvk.group, self.id.gvk.version)
        };

        Res {
            api_version,
            kind: self.id.gvk.kind.clone(),
            metadata: Metadata {
                name: self.id.name.clone(),
                namespace: self.id.namespace.clone(),
                rest: self.metadata.clone(),
            },
            manifest: self.manifest.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let res = Res::deserialize(deserializer)
            .map_err(|err| serde::de::Error::custom(format!("parsing resource: {err}")))?;

        let (group, version) = res
            .api_version
            .split_once('/')
            .map_or(("".into(), res.api_version.clone()), |(g, v)| {
                (g.into(), v.into())
            });

        let id = ResId {
            gvk: Gvk {
                group,
                version,
                kind: res.kind,
            },
            name: res.metadata.name,
            namespace: res.metadata.namespace,
        };

        Ok(Resource {
            id,
            metadata: res.metadata.rest,
            manifest: res.manifest,
        })
    }
}
