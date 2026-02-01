use core::fmt;
use json_patch::Patch as JsonPatch;
use regex::Regex;
use std::{path::PathBuf, sync::LazyLock};

use crate::{
    resource::{Metadata, Resource},
    yaml,
};
use compact_str::CompactString;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type Str = CompactString;

pub type Kustomization = Manifest<apiversion::V1Beta1, kind::Kustomize>;
pub type Component = Manifest<apiversion::V1Alpha1, kind::Component>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest<A, K> {
    #[serde(flatten)]
    pub type_meta: TypeMeta<A, K>,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[PathBuf]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub resources: Box<[PathBuf]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub patches: Box<[Patch]>,

    /// Legacy field, use `patches` instead.
    #[serde(
        rename = "patchesJson6902",
        default,
        skip_serializing_if = "<[_]>::is_empty"
    )]
    pub patches_json: Box<[JsonPatch6902]>,
    /// Legacy field, use `patches` instead.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub patches_strategic_merge: Box<[PathBuf]>,
    #[serde(
        default,
        skip_serializing_if = "<[_]>::is_empty",
        rename = "configMapGenerator"
    )]
    pub config_map_generators: Box<[Generator]>,
    #[serde(
        default,
        skip_serializing_if = "<[_]>::is_empty",
        rename = "secretGenerator"
    )]
    pub secret_generators: Box<[SecretGenerator]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub generators: Box<[PathBuf]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub transformers: Box<[PathBuf]>,
    #[serde(default)]
    pub generator_options: GeneratorOptions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Str>,
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub name_prefix: Str,
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub name_suffix: Str,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub labels: Box<[Label]>,
    /// Deprecated, use `labels` field instead.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub common_labels: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub common_annotations: IndexMap<Str, Str>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub images: Box<[ImageTag]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub replicas: Box<[Replica]>,
}

impl<A, K> Manifest<A, K> {
    pub fn transform_legacy_fields(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageTag {
    pub name: Str,
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub new_name: Str,
    // `new_tag` is the value used to replace the original tag.
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub new_tag: Str,
    // `digest` is the value used to replace the original image tag.
    // If `digest` is present `new_tag` is ignored.
    #[serde(default, skip_serializing_if = "Str::is_empty")]
    pub digest: Str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
pub struct SecretGenerator {
    #[serde(default, rename = "type")]
    pub ty: SecretType,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum SecretType {
    #[default]
    Opaque,
    // #[serde(rename = "kubernetes.io/tls")]
    // Tls,
}

impl fmt::Display for SecretType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opaque => write!(f, "Opaque"),
            // Self::Tls => write!(f, "kubernetes.io/tls"),
        }
    }
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

impl GeneratorOptions {
    pub fn static_default() -> &'static Self {
        static STATIC_DEFAULT: LazyLock<GeneratorOptions> = LazyLock::new(Default::default);
        &STATIC_DEFAULT
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyValuePairSources {
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub literals: Box<[KeyValuePair]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub files: Box<[MaybeKeyValuePair]>,
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub envs: Box<[PathBuf]>,
}

impl<'de> Deserialize<'de> for KeyValuePairSources {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Helper {
            #[serde(default)]
            literals: Box<[KeyValuePair]>,
            #[serde(default)]
            files: Box<[MaybeKeyValuePair]>,
            #[serde(default)]
            envs: Vec<PathBuf>,
            // Support for legacy singular `env` field
            env: Option<PathBuf>,
        }

        let mut helper = Helper::deserialize(deserializer)?;
        helper.envs.extend(helper.env);

        Ok(KeyValuePairSources {
            literals: helper.literals,
            files: helper.files,
            envs: helper.envs.into_boxed_slice(),
        })
    }
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
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
// Assuming inline patch is a JSON Patch or a file path for strategic merge patch, not sure if this
// matches kustomize's exact semantics.
pub enum Patch {
    OutOfLine {
        path: PathBuf,
        target: Option<Target>,
    },
    Json {
        #[serde(with = "crate::serde_ex::nested_yaml")]
        patch: JsonPatch,
        target: Target,
    },
    StrategicMerge {
        #[serde(with = "crate::serde_ex::nested_yaml")]
        patch: Resource,
        target: Option<Target>,
    },
}

// Untagged errors are too terrible to read, so we implement custom deserialization.
impl<'de> Deserialize<'de> for Patch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct OutOfLine {
            path: PathBuf,
            target: Option<Target>,
        }

        let value: json::Value = Deserialize::deserialize(deserializer)?;

        if let Some(obj) = value.as_object() {
            if obj.contains_key("path") {
                let helper =
                    json::from_value::<OutOfLine>(value).map_err(serde::de::Error::custom)?;
                return Ok(Patch::OutOfLine {
                    path: helper.path,
                    target: helper.target,
                });
            } else if let Some(patch_value) = obj.get("patch") {
                let target = if let Some(tv) = obj.get("target") {
                    Some(json::from_value::<Target>(tv.clone()).map_err(serde::de::Error::custom)?)
                } else {
                    None
                };

                let patch = patch_value
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("patch field must be a string "))?;

                // Try to deserialize as JsonPatch first
                match yaml::from_str::<JsonPatch>(patch) {
                    Ok(patch) => {
                        let target = target.ok_or_else(|| {
                            serde::de::Error::custom("target field is required for Json patches")
                        })?;

                        return Ok(Patch::Json { patch, target });
                    }
                    Err(json_err) => {
                        // Otherwise, try to deserialize as StrategicMerge patch
                        let patch = yaml::from_str::<Resource>(patch).map_err(|err| {
                            serde::de::Error::custom(format!(
                                "failed to deserialize patch as either JsonPatch due to `{json_err}`\nor StrategicMerge patch due to `{err}`",
                            ))
                        })?;
                        return Ok(Patch::StrategicMerge { patch, target });
                    }
                }
            }
        }

        Err(serde::de::Error::custom(format!(
            "invalid patch format: expected either `path: <path>` or inline Json/StrategicMerge patch, got `{value:?}`",
        )))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonPatch6902 {
    pub path: PathBuf,
    pub target: Target,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Target {
    LabelSelector(Selector),
    AnnotationSelector(Selector),
    #[serde(untagged)]
    Pattern(Pattern),
}

impl Target {
    pub fn matches(&self, resource: &Resource) -> bool {
        match self {
            Target::LabelSelector(_) | Target::AnnotationSelector(_) => {
                todo!("kube selectors not implemented yet")
            }
            Target::Pattern(pat) => pat.matches(resource),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pattern {
    #[serde(with = "crate::serde_ex::regex")]
    pub kind: Regex,
    #[serde(
        with = "crate::serde_ex::opt_regex",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<Regex>,
    #[serde(
        with = "crate::serde_ex::opt_regex",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub namespace: Option<Regex>,
}

impl Pattern {
    pub fn matches(&self, resource: &Resource) -> bool {
        resource.any_id_matches(|id| {
            self.kind.is_match(id.kind)
                && self.name.as_ref().is_none_or(|re| re.is_match(id.name))
                && self
                    .namespace
                    .as_ref()
                    .is_none_or(|re| id.namespace.is_some_and(|ns| re.is_match(ns)))
        })
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeMeta<V, K> {
    pub api_version: Option<V>,
    pub kind: Option<K>,
}

impl<V, K> Default for TypeMeta<V, K>
where
    V: Default,
    K: Default,
{
    fn default() -> Self {
        Self {
            api_version: Some(V::default()),
            kind: Some(K::default()),
        }
    }
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
}

pub mod kind {
    use super::define_symbol;

    define_symbol!(Kustomize = "Kustomization");
    define_symbol!(Component = "Component");
    define_symbol!(ResourceList = "ResourceList");
    define_symbol!(ImageTagTransformer = "ImageTagTransformer");
    define_symbol!(ServiceAccount = "ServiceAccount");
    define_symbol!(Namespace = "Namespace");
    define_symbol!(ConfigMapGenerator = "ConfigMapGenerator");
}

pub mod apiversion {
    use super::define_symbol;
    define_symbol!(V1Alpha1 = "kustomize.config.k8s.io/v1alpha1");
    define_symbol!(V1Beta1 = "kustomize.config.k8s.io/v1beta1");
    define_symbol!(Builtin = "builtin");
    define_symbol!(ConfigV1 = "config.kubernetes.io/v1");
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

        impl ::core::fmt::Display for $name {
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

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                other == $value
            }
        }
    };
}

use define_symbol;

pub trait Symbol: fmt::Debug + Send + Sync {
    const VALUE: &'static str;
}
