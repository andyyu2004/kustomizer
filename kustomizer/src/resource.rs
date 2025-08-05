mod view;

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use anyhow::ensure;
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
    id: ResId,
    root: serde_yaml::Mapping,
}

impl Resource {
    pub fn new(
        id: ResId,
        metadata: Metadata,
        mut root: serde_yaml::Mapping,
    ) -> anyhow::Result<Self> {
        ensure!(
            root.insert(
                serde_yaml::Value::String("metadata".into()),
                serde_yaml::to_value(&metadata)?,
            )
            .is_none(),
            "root must not duplicate metadata"
        );

        Ok(Resource { id, root })
    }

    pub fn id(&self) -> &ResId {
        &self.id
    }

    pub fn kind(&self) -> &Str {
        &self.id.kind
    }

    pub fn root(&self) -> &serde_yaml::Mapping {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut serde_yaml::Mapping {
        &mut self.root
    }
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
        skip_serializing_if = "Option::is_none"
    )]
    pub behavior: Option<Behavior>,
    #[serde(flatten)]
    pub rest: IndexMap<Str, Str>,
}

impl Annotations {
    pub fn is_empty(&self) -> bool {
        self.function_spec.is_none() && self.behavior.is_none() && self.rest.is_empty()
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Res {
    api_version: Str,
    kind: Str,
    metadata: Metadata,
    #[serde(flatten)]
    root: serde_yaml::Mapping,
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

        debug_assert!(
            self.root.contains_key("metadata"),
            "Resource root must contain metadata"
        );

        let mut root = self.root.clone();
        let metadata = root.remove("metadata").unwrap();

        let metadata = serde_yaml::from_value(metadata).expect("invalid metadata");

        Res {
            api_version,
            kind: self.kind().clone(),
            metadata,
            root,
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

        Resource::new(id, res.metadata, res.root).map_err(serde::de::Error::custom)
    }
}
