use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use compact_str::format_compact;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::manifest::{Behavior, FunctionSpec, Str};

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
            write!(f, "{}.{}", self.kind, self.version)
        } else {
            write!(f, "{}.{}.{}", self.kind, self.version, self.group)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResId {
    #[serde(flatten)]
    pub gvk: Gvk,
    pub name: Str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Str>,
}

impl Deref for ResId {
    type Target = Gvk;

    fn deref(&self) -> &Self::Target {
        &self.gvk
    }
}

impl fmt::Debug for ResId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for ResId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}/{}.{namespace}", self.gvk, self.name)?;
        } else {
            write!(f, "{}/{}", self.gvk, self.name)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: ResId,
    pub metadata: Metadata,
    pub root: serde_yaml::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Res {
    api_version: Str,
    kind: Str,
    metadata: Metadata,
    #[serde(flatten)]
    root: serde_yaml::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub name: Str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Str>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub labels: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "Annotations::is_empty")]
    pub annotations: Annotations,
    #[serde(flatten)]
    pub rest: IndexMap<Str, serde_yaml::Value>,
}

impl Deref for Metadata {
    type Target = IndexMap<Str, serde_yaml::Value>;

    fn deref(&self) -> &Self::Target {
        &self.rest
    }
}

impl DerefMut for Metadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Annotations {
    #[serde(
        rename = "config.kubernetes.io/function",
        with = "crate::serde_ex::nested_yaml",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub function_spec: Option<FunctionSpec>,
    #[serde(
        rename = "kustomize.config.k8s.io/behavior",
        default,
        skip_serializing_if = "Behavior::is_default"
    )]
    pub behavior: Behavior,
    #[serde(flatten)]
    pub rest: IndexMap<Str, Str>,
}

impl Annotations {
    pub fn is_empty(&self) -> bool {
        self.function_spec.is_none() && self.rest.is_empty()
    }
}

impl Deref for Annotations {
    type Target = IndexMap<Str, Str>;

    fn deref(&self) -> &Self::Target {
        &self.rest
    }
}

impl DerefMut for Annotations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rest
    }
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
            metadata: self.metadata.clone(),
            root: self.root.clone(),
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
            name: res.metadata.name.clone(),
            namespace: res.metadata.namespace.clone(),
        };

        Ok(Resource {
            id,
            metadata: res.metadata,
            root: res.root,
        })
    }
}
