use core::fmt;
use json_patch::Patch as JsonPatch;
use std::path::PathBuf;

use crate::{fieldspec, resource::Metadata};
use compact_str::CompactString;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type Str = CompactString;

pub type Kustomization = Manifest<apiversion::Beta, kind::Kustomize>;
pub type Component = Manifest<apiversion::Alpha, kind::Component>;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest<A, K> {
    #[serde(flatten)]
    pub type_meta: TypeMeta<A, K>,
    #[serde(default)]
    pub metadata: Metadata,
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
    #[serde(
        default,
        skip_serializing_if = "<[_]>::is_empty",
        rename = "configMapGenerator"
    )]
    pub config_map_generators: Box<[Generator]>,
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
    #[serde(default)]
    pub options: GeneratorOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FunctionSpec {
    Exec(ExecSpec),
    Container(ContainerSpec),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecSpec {
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub args: Box<[Str]>,
    // TODO this is passed with key=value syntax
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<Str, Str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContainerSpec {
    pub image: Str,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<Str, Str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct GeneratorOptions {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub labels: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub annotations: IndexMap<Str, Str>,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Behavior {
    #[default]
    Create,
    Merge,
    Replace,
}

impl fmt::Display for Behavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Create => write!(f, "create"),
            Self::Merge => write!(f, "merge"),
            Self::Replace => write!(f, "replace"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Replica {
    pub name: Str,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
// Assuming inline patch is a JSON Patch or a file path for strategic merge patch, not sure if this
// matches kustomize's exact semantics.
pub enum Patch {
    Json {
        #[serde(with = "crate::serde_ex::nested_yaml")]
        patch: JsonPatch,
        target: Target,
    },
    StrategicMerge {
        path: PathBuf,
        target: Option<Target>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Target {
    LabelSelector(Selector),
    AnnotationSelector(Selector),
    #[serde(untagged)]
    Pattern(Pattern),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Pattern {
    pub kind: Str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Label {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub pairs: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub include_selectors: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub include_templates: bool,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub field_specs: Box<[fieldspec::FieldSpec]>,
}

pub mod kind {
    use super::define_symbol;

    define_symbol!(Kustomize = "Kustomization");
    define_symbol!(Component = "Component");
    define_symbol!(ResourceList = "ResourceList");
}

pub mod apiversion {
    use super::define_symbol;
    define_symbol!(Alpha = "kustomize.config.k8s.io/v1alpha1");
    define_symbol!(Beta = "kustomize.config.k8s.io/v1beta1");
}

macro_rules! define_symbol {
    ($name:ident = $value:literal) => {
        #[derive(Clone, PartialEq, Eq, Hash, Default)]
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
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
                let value: $crate::manifest::Str = ::serde::Deserialize::deserialize(deserializer)?;
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

        impl $crate::manifest::Symbol for $name {
            const VALUE: &'static str = $value;
        }
    };
}

use define_symbol;

pub trait Symbol: fmt::Debug + Send + Sync {
    const VALUE: &'static str;
}
