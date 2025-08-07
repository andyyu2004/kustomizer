mod view;

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use anyhow::{Context, ensure};
use compact_str::format_compact;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
    PathExt, PathId,
    manifest::{Behavior, FunctionSpec, Str},
};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct GvkMatcher {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<Str>,
}

impl fmt::Display for GvkMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(kind) = &self.kind {
            write!(f, "{kind}.")?;
        }

        if let Some(version) = &self.version {
            write!(f, "{version}.")?;
        }

        if let Some(group) = &self.group {
            write!(f, "{group}")
        } else {
            write!(f, "*")
        }
    }
}

impl GvkMatcher {
    pub fn matches(&self, gvk: &Gvk) -> bool {
        (self.group.is_none() || self.group.as_ref() == Some(&gvk.group))
            && (self.version.is_none() || self.version.as_ref() == Some(&gvk.version))
            && (self.kind.is_none() || self.kind.as_ref() == Some(&gvk.kind))
    }

    pub fn overlaps_with(&self, other: &GvkMatcher) -> bool {
        (self.group.is_none() || other.group.is_none() || self.group == other.group)
            && (self.version.is_none() || other.version.is_none() || self.version == other.version)
            && (self.kind.is_none() || other.kind.is_none() || self.kind == other.kind)
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
    root: AnyObject,
}

pub type AnyObject = serde_json::Map<String, serde_json::Value>;

impl Resource {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let id = PathId::make(path)
            .with_context(|| format!("loading resource from path {}", path.pretty()))?;
        let file = std::fs::File::open(id)?;
        Ok(serde_yaml::from_reader(file)?)
    }

    pub fn new(id: ResId, metadata: Metadata, mut root: AnyObject) -> anyhow::Result<Self> {
        ensure!(
            root.insert("metadata".into(), serde_json::to_value(&metadata)?,)
                .is_none(),
            "root must not duplicate metadata"
        );

        Ok(Resource { id, root })
    }

    pub fn id(&self) -> &ResId {
        &self.id
    }

    pub fn name(&self) -> &Str {
        &self.id.name
    }

    pub fn namespace(&self) -> Option<&Str> {
        self.id.namespace.as_ref()
    }

    pub fn gvk(&self) -> &Gvk {
        &self.id.gvk
    }

    pub fn api_version(&self) -> &Str {
        &self.id.gvk.version
    }

    pub fn kind(&self) -> &Str {
        &self.id.kind
    }

    pub fn root(&self) -> &AnyObject {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut AnyObject {
        &mut self.root
    }

    pub fn patch(&mut self, patch: Self) -> anyhow::Result<()> {
        crate::patch::apply(self, patch)
            .with_context(|| format!("applying patch to resource `{}`", self.id))
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
    root: AnyObject,
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

        let metadata = serde_json::from_value(metadata).expect("invalid metadata");

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
