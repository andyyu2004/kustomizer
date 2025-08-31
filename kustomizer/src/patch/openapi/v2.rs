use std::{collections::HashSet, fmt, str::FromStr, sync::OnceLock};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{manifest::Str, resource::Gvk};

const SPEC_V2_GZ: &[u8] = include_bytes!("./openapi-v2-kubernetes-1.32-minimized.json");

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    definitions: IndexMap<TypeMeta, Schema>,
    paths: IndexMap<String, Path>,
    #[serde(skip)]
    namespaced: OnceLock<HashSet<TypeMeta>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Path {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<Operation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Operation {
    #[serde(
        rename = "x-kubernetes-group-version-kind",
        skip_serializing_if = "Option::is_none"
    )]
    kubernetes_gvk: Option<Gvk>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TypeMeta {
    api_version: Str,
    kind: Str,
}

impl Serialize for TypeMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for TypeMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)
            .map_err(|err| serde::de::Error::custom(format!("parsing DefinitionId: {err}")))?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for TypeMeta {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (s, kind) = s
            .rsplit_once('.')
            .ok_or_else(|| anyhow::anyhow!("missing kind: {s}"))?;

        // TODO group has an extra prefix such as "io.k8s.api." in the spec.
        // We only use apiVersion and kind to match resources, so we can ignore it for now.
        let (_group, api_version) = match s.rsplit_once(".") {
            Some((group, api_version)) => (group, api_version),
            None => ("", s),
        };

        Ok(Self {
            api_version: api_version.into(),
            kind: kind.into(),
        })
    }
}

impl fmt::Display for TypeMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.api_version, self.kind)
    }
}

impl Spec {
    pub fn load() -> &'static Self {
        static CACHE: OnceLock<Spec> = OnceLock::new();
        CACHE.get_or_init(|| {
            serde_json::from_reader(SPEC_V2_GZ).expect("test should guarantee this is valid")
        })
    }

    pub fn is_namespaced(&self, gvk: &Gvk) -> bool {
        let namespaced = self.namespaced.get_or_init(|| {
            let mut set = HashSet::new();
            for (route, path) in &self.paths {
                let Some(gvk) = path.get.as_ref().and_then(|op| op.kubernetes_gvk.as_ref()) else {
                    continue;
                };

                if route.contains("/namespaces/{namespace}/") {
                    set.insert(TypeMeta {
                        api_version: gvk.version.clone(),
                        kind: gvk.kind.clone(),
                    });
                }
            }

            set
        });

        let type_meta = TypeMeta {
            api_version: gvk.version.clone(),
            kind: gvk.kind.clone(),
        };

        namespaced.contains(&type_meta)
        // assume if we have no definition for the type, it is namespaced.
        || !self.definitions.contains_key(&type_meta)
    }

    pub fn schema_for(&self, gvk: &Gvk) -> Option<&ObjectSchema> {
        let definition_id = TypeMeta {
            api_version: gvk.version.clone(),
            kind: gvk.kind.clone(),
        };
        self.definitions
            .get(&definition_id)
            .map(|schema| match &schema.ty {
                Some(Type::Object(schema)) => schema,
                _ => panic!(
                    "expected ObjectSchema for {definition_id}, found: {:?}",
                    schema.ty
                ),
            })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    #[serde(flatten)]
    pub ty: Option<Type>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlineOrRef<T> {
    Ref(Ref),
    Inline(T),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ref {
    #[serde(rename = "$ref")]
    reference: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectSchema {
    pub properties: IndexMap<String, Schema>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Type {
    Object(ObjectSchema),
    Array { items: InlineOrRef<Box<Schema>> },
    String,
    Integer,
    Boolean,
}

#[cfg(test)]
mod tests {
    use super::Spec;

    #[test]
    #[ignore]
    fn minimize_openapi_v2_spec() -> anyhow::Result<()> {
        let reader = flate2::read::GzDecoder::new(
            include_bytes!("./openapi-v2-kubernetes-1.32.json.gz").as_ref(),
        );

        let spec: Spec = serde_json::from_reader(reader)?;

        let file = std::fs::File::create(
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json.tmp",
        )?;
        serde_json::to_writer(file, &spec)?;

        std::fs::rename(
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json.tmp",
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json",
        )?;

        // Ensure the spec can be loaded
        super::Spec::load();

        Ok(())
    }

    #[test]
    fn check_openapi_spec() {
        super::Spec::load();
    }
}
