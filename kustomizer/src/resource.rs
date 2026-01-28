mod refs;
mod shorthash;
mod view;

use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

use anyhow::{Context, ensure};
use compact_str::format_compact;
use dashmap::{DashMap, Entry};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
    PathExt, PathId,
    manifest::{Behavior, FunctionSpec, Str},
    patch::merge_patch,
};

pub use self::refs::RefSpecs;

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
            write!(f, "{}.{}", self.version, self.kind)
        } else {
            write!(f, "{}.{}.{}", self.group, self.version, self.kind)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct GvkMatcher {
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub group: Str,
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub version: Str,
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub kind: Str,
}

impl fmt::Display for GvkMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.kind.is_empty() {
            write!(f, "{}.", self.kind)?;
        }

        if !self.version.is_empty() {
            write!(f, "{}.", self.version)?;
        }

        if !self.group.is_empty() {
            write!(f, "{}", self.group)
        } else {
            write!(f, "*")
        }
    }
}

impl GvkMatcher {
    pub fn matches(&self, gvk: &Gvk) -> bool {
        (self.group.is_empty() || self.group == gvk.group)
            && (self.version.is_empty() || self.version == gvk.version)
            && (self.kind.is_empty() || self.kind == gvk.kind)
    }

    pub fn overlaps_with(&self, other: &GvkMatcher) -> bool {
        (self.group.is_empty() || other.group.is_empty() || self.group == other.group)
            && (self.version.is_empty()
                || other.version.is_empty()
                || self.version == other.version)
            && (self.kind.is_empty() || other.kind.is_empty() || self.kind == other.kind)
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

pub(crate) struct ResIdRef<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub namespace: Option<&'a str>,
}

impl fmt::Debug for ResIdRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(namespace) = &self.namespace {
            write!(f, "{}/{}.{namespace}", self.kind, self.name)?;
        } else {
            write!(f, "{}/{}", self.kind, self.name)?;
        }
        Ok(())
    }
}

impl PartialEq<ResIdRef<'_>> for ResId {
    fn eq(&self, other: &ResIdRef<'_>) -> bool {
        self.kind == other.kind
            && self.name == other.name
            && self.namespace.as_deref() == other.namespace
    }
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
    // This will always be a JSON object, but stored as serde_json::Value for impl convenience
    root: serde_json::Value,
}

pub type Object = serde_json::Map<String, serde_json::Value>;

static RES_CACHE: LazyLock<DashMap<PathId, Resource>> = LazyLock::new(Default::default);

impl Resource {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let path = PathId::make(path)
            .with_context(|| format!("loading resource from path {}", path.pretty()))?;

        match RES_CACHE.entry(path) {
            Entry::Occupied(e) => Ok(e.get().clone()),
            Entry::Vacant(e) => {
                let file = std::fs::File::open(path)?;
                let resource = serde_yaml::from_reader(file)?;
                Ok(e.insert(resource).value().clone())
            }
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.id.kind == "Dummy" && self.id.name == "dummy" && self.id.namespace.is_none()
    }

    pub fn dummy() -> Self {
        Resource {
            id: ResId {
                gvk: Gvk {
                    group: Default::default(),
                    version: Default::default(),
                    kind: "Dummy".into(),
                },
                name: "dummy".into(),
                namespace: Default::default(),
            },
            root: serde_json::json!({
                "metadata": {
                    "name": "dummy"
                }
            }),
        }
    }

    pub fn new(id: ResId, metadata: Metadata, mut root: Object) -> anyhow::Result<Self> {
        assert_eq!(id.name, metadata.name, "id.name must match metadata.name");
        assert_eq!(
            id.namespace, metadata.namespace,
            "id.namespace must match metadata.namespace"
        );

        ensure!(
            root.insert("metadata".into(), serde_json::to_value(&metadata)?)
                .is_none(),
            "root must not duplicate metadata"
        );

        assert!(!id.name.contains('/'), "name must not contain '/'");
        assert!(!id.name.contains(','), "name must not contain ','");

        if let Some(ns) = &id.namespace {
            assert!(!ns.contains('/'), "namespace must not contain '/'");
            assert!(!ns.contains(','), "namespace must not contain ','");
        }

        assert!(!id.kind.contains('/'), "kind must not contain '/'");
        assert!(!id.kind.contains(','), "kind must not contain ','");

        Ok(Resource {
            id,
            root: serde_json::Value::Object(root),
        })
    }

    pub fn from_parts(id: ResId, mut root: Object) -> anyhow::Result<Self> {
        let metadata = match root.remove("metadata") {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| anyhow::anyhow!("invalid metadata: {}", e))?,
            None => Metadata::default(),
        };

        Resource::new(id, metadata, root)
    }

    #[must_use]
    pub(crate) fn into_parts(self) -> (ResId, Object) {
        let Resource { id, root } = self;
        (
            id,
            match root {
                serde_json::Value::Object(map) => map,
                _ => panic!("root is always an object"),
            },
        )
    }

    #[must_use]
    pub fn with_name(mut self, name: Str) -> Self {
        self.store_curr_id();
        self.make_metadata_mut().set_name(name.clone());
        let (mut id, root) = self.into_parts();
        id.name = name;
        Self::from_parts(id, root).expect("invariants should be maintained by this function")
    }

    #[must_use]
    pub fn with_namespace(mut self, ns: Option<Str>) -> Self {
        self.store_curr_id();
        self.make_metadata_mut().set_namespace(ns.clone());
        let (mut id, root) = self.into_parts();
        id.namespace = ns;
        Self::from_parts(id, root).expect("invariants should be maintained by this function")
    }

    pub fn id(&self) -> &ResId {
        &self.id
    }

    fn store_curr_id(&mut self) {
        let id = self.id().clone();
        self.make_metadata_mut()
            .make_annotations_mut()
            .insert_or_update(annotation::PREVIOUS_NAMES, |s| {
                s.push_str(&format!("{},", id.name))
            });

        self.make_metadata_mut()
            .make_annotations_mut()
            .insert_or_update(annotation::PREVIOUS_NAMESPACES, |s| {
                let ns = id.namespace.as_deref().unwrap_or("");
                s.push_str(&format!("{ns},"))
            });

        self.make_metadata_mut()
            .make_annotations_mut()
            .insert_or_update(annotation::PREVIOUS_KINDS, |s| {
                s.push_str(&format!("{},", id.kind))
            });
    }

    pub(crate) fn any_id_matches(&self, p: impl FnMut(ResIdRef<'_>) -> bool) -> bool {
        self.all_ids().any(p)
    }

    /// Iterator over all names this resource has had, including current and previous names.
    pub(crate) fn all_ids(&self) -> impl Iterator<Item = ResIdRef<'_>> + fmt::Debug {
        let curr = std::iter::once(ResIdRef {
            kind: &self.id.kind,
            name: &self.id.name,
            namespace: self.id.namespace.as_deref(),
        });

        let prev_names = self
            .metadata()
            .and_then(|md| md.annotations())
            .and_then(|a| a.get(annotation::PREVIOUS_NAMES))
            .unwrap_or_default()
            .split(',');

        let prev_namespaces = self
            .metadata()
            .and_then(|md| md.annotations())
            .and_then(|a| a.get(annotation::PREVIOUS_NAMESPACES))
            .unwrap_or_default()
            .split(',');

        let prev_kinds = self
            .metadata()
            .and_then(|md| md.annotations())
            .and_then(|a| a.get(annotation::PREVIOUS_KINDS))
            .unwrap_or_default()
            .split(',');

        debug_assert_eq!(
            prev_namespaces.clone().count(),
            prev_names.clone().count(),
            "previous names and namespaces must have same length: {:?} and {:?}",
            prev_namespaces.collect::<Vec<_>>(),
            prev_names.collect::<Vec<_>>(),
        );

        debug_assert_eq!(
            prev_kinds.clone().count(),
            prev_names.clone().count(),
            "previous names and kinds must have same length: {:?} and {:?}",
            prev_kinds.collect::<Vec<_>>(),
            prev_names.collect::<Vec<_>>(),
        );

        curr.chain(prev_names.zip(prev_namespaces).zip(prev_kinds).map(
            |((name, namespace), kind)| ResIdRef {
                kind,
                name,
                namespace: if namespace.is_empty() {
                    None
                } else {
                    Some(namespace)
                },
            },
        ))
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

    pub fn root(&self) -> &Object {
        self.root.as_object().expect("root is always an object")
    }

    pub fn root_mut(&mut self) -> &mut Object {
        self.root.as_object_mut().expect("root is always an object")
    }

    pub(crate) fn root_raw_mut(&mut self) -> &mut serde_json::Value {
        &mut self.root
    }

    pub fn patch(&mut self, patch: Self) -> anyhow::Result<()> {
        merge_patch(self, patch)
            .with_context(|| format!("applying patch to resource `{}`", self.id))
    }

    /// Right-biased merge of metadata fields (labels and annotations) into `self`.
    /// The resource's identity (name/namespace) is preserved.
    pub(crate) fn merge_metadata(&mut self, other: &Self) -> anyhow::Result<()> {
        let mut metadata = self.make_metadata_mut();

        if let Some(other_labels) = other.metadata().and_then(|md| md.labels()) {
            let mut labels = metadata.make_labels_mut();
            for (key, value) in other_labels.iter() {
                labels.insert(key, value);
            }
        }

        if let Some(other_annotations) = other.metadata().and_then(|md| md.annotations()) {
            let mut annotations = metadata.make_annotations_mut();
            for (key, value) in other_annotations.iter() {
                // Skip internal annotations that should not be merged
                if key == annotation::FUNCTION
                    || key == annotation::BEHAVIOR
                    || key == annotation::NEEDS_HASH
                    || key == annotation::PREVIOUS_NAMES
                    || key == annotation::PREVIOUS_NAMESPACES
                    || key == annotation::PREVIOUS_KINDS
                {
                    continue;
                }
                annotations.insert(key, value);
            }
        }

        Ok(())
    }

    // right-biased merge of data fields `data` and `binaryData` and `stringData`
    pub(crate) fn merge_data_fields(&mut self, other: Self) -> anyhow::Result<()> {
        // TODO merging metadata and annotations, not sure what is correct behavior for this?

        // Merge `data` field
        let left_data = self
            .root_mut()
            .get_mut("data")
            .and_then(|data| data.as_object_mut());
        let right_data = other.root().get("data").and_then(|data| data.as_object());

        match (left_data, right_data) {
            (Some(left), Some(right)) => {
                for (key, value) in right {
                    left.insert(key.clone(), value.clone());
                }
            }
            (None, Some(right)) => {
                if !right.is_empty() {
                    self.root_mut()
                        .insert("data".into(), serde_json::Value::Object(right.clone()));
                }
            }
            (_, None) => {}
        }

        // Merge `binaryData` field
        let left_binary_data = self
            .root_mut()
            .get_mut("binaryData")
            .and_then(|data| data.as_object_mut());

        let right_binary_data = other
            .root()
            .get("binaryData")
            .and_then(|data| data.as_object());

        match (left_binary_data, right_binary_data) {
            (Some(left), Some(right)) => {
                for (key, value) in right {
                    left.insert(key.clone(), value.clone());
                }
            }
            (None, Some(right)) => {
                if !right.is_empty() {
                    self.root_mut().insert(
                        "binaryData".into(),
                        serde_json::Value::Object(right.clone()),
                    );
                }
            }
            (_, None) => {}
        }

        if self.root().get("stringData").is_some() || other.root().get("stringData").is_some() {
            anyhow::bail!(
                "merging configmaps with `stringData` is not supported, kustomize has strange behavior for this so this is disallowed, use `data` instead"
            );
        }

        Ok(())
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
    root: Object,
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
            self.root().contains_key("metadata"),
            "Resource root must contain metadata"
        );

        let mut root = self.root().clone();
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

pub(crate) mod annotation {
    pub const FUNCTION: &str = "config.kubernetes.io/function";
    pub const BEHAVIOR: &str = "kustomize.config.k8s.io/behavior";
    pub const NEEDS_HASH: &str = "kustomize.config.k8s.io/needs-hash";

    pub const PREVIOUS_NAMES: &str = "internal.config.kubernetes.io/previous-names";
    pub const PREVIOUS_NAMESPACES: &str = "internal.config.kubernetes.io/previous-namespaces";
    pub const PREVIOUS_KINDS: &str = "internal.config.kubernetes.io/previous-kinds";
}
