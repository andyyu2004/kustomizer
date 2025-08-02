use core::fmt;
use std::path::PathBuf;

use compact_str::CompactString;
use indexmap::IndexMap;
use serde::Deserialize;

type Str = CompactString;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kustomization {
    #[serde(flatten)]
    type_meta: TypeMeta<KustomizeAPIVersion, KustomizeKind>,
    metadata: ObjectMeta,
    #[serde(default)]
    namespace: Option<Str>,
    #[serde(default)]
    components: Box<[PathBuf]>,
    #[serde(default)]
    resources: Box<[PathBuf]>,
    name_prefix: Option<Str>,
    name_suffix: Option<Str>,
    #[serde(default)]
    labels: Box<[Label]>,
    #[serde(default)]
    common_annotations: IndexMap<Str, Str>,
    patches_strategic_merge: Box<[PathBuf]>,
    // patches_json6902: Box<[PathBuf]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeMeta<V: Symbol, K: Symbol> {
    pub api_version: V,
    pub kind: K,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ObjectMeta {
    pub name: Str,
    pub namespace: Option<Str>,
    #[serde(default)]
    pub labels: IndexMap<Str, Str>,
    #[serde(default)]
    pub annotations: IndexMap<Str, Str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    #[serde(default)]
    pub pairs: IndexMap<Str, Str>,
    pub include_selectors: bool,
}

define_symbol!(KustomizeAPIVersion = "kustomize.config.k8s.io/v1beta1");
define_symbol!(KustomizeKind = "Kustomization");

macro_rules! define_symbol {
    ($name:ident = $value:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl<'de> Deserialize<'de> for $name {
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
