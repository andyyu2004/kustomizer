use core::fmt;
use json_patch::Patch as JsonPatch;
use std::path::PathBuf;

use compact_str::CompactString;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type Str = CompactString;

pub type Kustomization = Manifest<KustomizeBeta, KustomizeKind>;
pub type Component = Manifest<KustomizeAlpha, ComponentKind>;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest<A, K> {
    #[serde(flatten)]
    pub type_meta: TypeMeta<A, K>,
    #[serde(default)]
    pub metadata: ObjectMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Str>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[PathBuf]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub resources: Box<[PathBuf]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_prefix: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_suffix: Option<Str>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub labels: Box<[Label]>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub common_annotations: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub patches: Box<[Patch]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub replicas: Box<[Replica]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub config_map_generator: Box<[Generator]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub generators: Box<[PathBuf]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub transformers: Box<[PathBuf]>,
    #[serde(default)]
    pub generator_options: GeneratorOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Generator {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Str>,
    pub name: Str,
    #[serde(default)]
    pub behavior: Behavior,
    #[serde(flatten)]
    pub sources: KeyValuePairSources,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct GeneratorOptions {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    labels: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    annotations: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_name_suffix_hash: Option<bool>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub immutable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyValuePairSources {
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub literals: Box<[KeyValuePair]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub files: Box<[MaybeKeyValuePair]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaybeKeyValuePair {
    pub key: Option<Str>,
    pub value: Str,
}

impl Serialize for MaybeKeyValuePair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut value = self.value.clone();
        if let Some(key) = &self.key {
            value.insert_str(0, &format!("{}=", key));
        }
        serializer.serialize_str(&value)
    }
}

impl<'de> Deserialize<'de> for MaybeKeyValuePair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Str = Deserialize::deserialize(deserializer)?;
        let mut parts = value.splitn(2, '=');
        let fst = parts.next().map(|s| s.into());
        match parts.next() {
            Some(val) => Ok(MaybeKeyValuePair {
                key: fst,
                value: val.into(),
            }),
            _ => Ok(MaybeKeyValuePair {
                key: None,
                value: fst.unwrap_or_default(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValuePair {
    pub key: Str,
    pub value: Str,
}

impl Serialize for KeyValuePair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = format!("{}={}", self.key, self.value);
        serializer.serialize_str(&value)
    }
}

impl<'de> Deserialize<'de> for KeyValuePair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let maybe_pair: MaybeKeyValuePair = Deserialize::deserialize(deserializer)?;
        Ok(KeyValuePair {
            key: maybe_pair.key.ok_or_else(|| {
                serde::de::Error::custom("missing key, must be in the format `<key>=<value>`")
            })?,
            value: maybe_pair.value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Behavior {
    #[default]
    Create,
    Merge,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Replica {
    pub name: Str,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Patch {
    Json {
        #[serde(flatten)]
        patch: PathOrInlinePatch,
        target: Target,
    },
    StrategicMerge {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum PathOrInlinePatch {
    #[serde(rename = "path")]
    Path(PathBuf),
    #[serde(rename = "patch", with = "nested_yaml")]
    Inline(JsonPatch),
}

mod nested_yaml {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(patch: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: ToString,
    {
        patch.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let s: Str = Deserialize::deserialize(deserializer)?;
        let yaml =
            serde_yaml::from_str::<serde_yaml::Value>(&s).map_err(serde::de::Error::custom)?;
        T::deserialize(yaml).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Target {
    LabelSelector(Selector),
    AnnotationSelector(Selector),
    #[serde(untagged)]
    Id(ResId),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Gvk {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Str>,
    pub kind: Str,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_cluster_scoped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResId {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Str>,
    pub kind: Str,
    pub name: Str,
    pub namespace: Option<Str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    tmp_unparsed: Str,
}

impl Serialize for Selector {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Placeholder implementation, actual logic to serialize selectors should be added
        _serializer.serialize_str(&self.tmp_unparsed)
    }
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Placeholder implementation, actual logic to deserialize selectors should be added
        let tmp_unparsed: Str = Deserialize::deserialize(_deserializer)?;
        Ok(Selector { tmp_unparsed })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeMeta<V, K> {
    pub api_version: Option<V>,
    pub kind: Option<K>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ObjectMeta {
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub name: Str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Str>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub labels: IndexMap<Str, Str>,
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub annotations: IndexMap<Str, Str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Label {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub pairs: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub include_selectors: bool,
}

define_symbol!(KustomizeAlpha = "kustomize.config.k8s.io/v1alpha1");
define_symbol!(KustomizeBeta = "kustomize.config.k8s.io/v1beta1");
define_symbol!(KustomizeKind = "Kustomization");
define_symbol!(ComponentKind = "Component");

macro_rules! define_symbol {
    ($name:ident = $value:literal) => {
        #[derive(Clone, PartialEq, Eq, Hash, Default)]
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", $value)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str($value)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value: Str = Deserialize::deserialize(deserializer)?;
                if value == $value {
                    Ok($name)
                } else {
                    Err(serde::de::Error::custom(format!(
                        "expected `{}`, found `{value}`",
                        $value
                    )))
                }
            }
        }

        impl Symbol for $name {
            const VALUE: &'static str = $value;
        }
    };
}

use define_symbol;

pub trait Symbol: fmt::Debug {
    const VALUE: &'static str;
}
