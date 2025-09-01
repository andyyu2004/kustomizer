use std::{collections::HashSet, fmt, str::FromStr, sync::OnceLock};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize, ser::SerializeStruct as _};

use crate::{
    manifest::Str,
    patch::{ListType, PatchStrategy},
    resource::{Gvk, Object},
};

const SPEC_V2_GZ: &[u8] = include_bytes!("./openapi-v2-kubernetes-1.32-minimized.json");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    definitions: IndexMap<TypeMeta, Type>,
    paths: IndexMap<String, Path>,
    #[serde(skip)]
    namespaced: OnceLock<HashSet<TypeMeta>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Path {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<Operation>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Operation {
    #[serde(
        rename = "x-kubernetes-group-version-kind",
        skip_serializing_if = "Option::is_none"
    )]
    gvk: Option<Gvk>,
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
                let Some(gvk) = path.get.as_ref().and_then(|op| op.gvk.as_ref()) else {
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

    pub fn schema_for(&self, gvk: &Gvk) -> Option<&ObjectType> {
        let definition_id = TypeMeta {
            api_version: gvk.version.clone(),
            kind: gvk.kind.clone(),
        };
        self.definitions
            .get(&definition_id)
            .map(|schema| match &schema {
                Type::Object(schema) => schema,
                _ => panic!(
                    "expected ObjectSchema for {definition_id}, found: {:?}",
                    schema
                ),
            })
    }

    pub(crate) fn resolve<'a>(&'a self, schema: &'a InlineOrRef<Box<Type>>) -> &'a Type {
        match schema {
            InlineOrRef::Inline(ty) => ty,
            InlineOrRef::Ref(r) => self
                .definitions
                .get(&r.reference)
                .expect("spec should have definition for $ref"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlineOrRef<T> {
    Ref(Ref),
    Inline(T),
}

#[derive(Debug, PartialEq)]
pub struct Ref {
    reference: TypeMeta,
}

impl FromStr for Ref {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#/definitions/") {
            return Err(anyhow::anyhow!(
                "invalid $ref, must start with '#/definitions/': {s}"
            ));
        }

        let reference = TypeMeta::from_str(&s["#/definitions/".len()..])?;

        Ok(Self { reference })
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#/definitions/{}", self.reference)
    }
}

impl Serialize for Ref {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("$ref", 1)?;
        state.serialize_field("$ref", &self.to_string())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Ref {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RefHelper {
            #[serde(rename = "$ref")]
            reference: String,
        }

        let helper = RefHelper::deserialize(deserializer)?;
        Self::from_str(&helper.reference).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectType {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, InlineOrRef<Box<Type>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<InlineOrRef<Box<Type>>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub items: InlineOrRef<Box<Type>>,

    #[serde(
        rename = "x-kubernetes-patch-strategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub patch_strategy: Option<PatchStrategy>,

    #[serde(
        rename = "x-kubernetes-list-type",
        skip_serializing_if = "Option::is_none"
    )]
    pub list_type: Option<ListType>,

    #[serde(
        rename = "x-kubernetes-list-map-keys",
        skip_serializing_if = "Option::is_none"
    )]
    pub list_map_keys: Option<Box<[Str]>>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Type {
    Object(ObjectType),
    Array(ArrayType),
    String,
    Integer,
    Number,
    Boolean,
    Any,
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct Helper {
            // If there is no `type` field, it indicates an `any` type.
            // It's also a deserialization nightmare for serde so we need a helper struct for
            // deserializing this.
            #[serde(rename = "type")]
            kind: Option<TypeKind>,
            #[serde(flatten)]
            rest: Object,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        enum TypeKind {
            Object,
            Array,
            String,
            Integer,
            Number,
            Boolean,
            Any,
        }

        let ty = Helper::deserialize(deserializer)?;

        let ty = match ty.kind {
            Some(TypeKind::Object) => {
                serde_json::from_value::<ObjectType>(serde_json::Value::Object(ty.rest.clone()))
                    .map(Type::Object)
                    .map_err(|err| {
                        serde::de::Error::custom(format!(
                            "parsing ObjectType: {}\n{err}",
                            serde_json::to_string_pretty(&ty.rest).unwrap()
                        ))
                    })?
            }
            Some(TypeKind::Array) => {
                serde_json::from_value::<ArrayType>(serde_json::Value::Object(ty.rest))
                    .map(Type::Array)
                    .map_err(|err| serde::de::Error::custom(format!("parsing ArrayType: {err}")))?
            }
            Some(TypeKind::Number) => Type::Number,
            Some(TypeKind::String) => Type::String,
            Some(TypeKind::Integer) => Type::Integer,
            Some(TypeKind::Boolean) => Type::Boolean,
            Some(TypeKind::Any) | None => Type::Any,
        };
        Ok(ty)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use serde_json::json;

    use crate::patch::openapi::v2::ObjectType;

    use super::{Spec, Type};

    #[test]
    fn test_serde() -> anyhow::Result<()> {
        assert_eq!(serde_json::from_value::<Type>(json!({}))?, Type::Any);
        assert_eq!(
            serde_json::from_value::<Type>(json!({"type": "string"}))?,
            Type::String
        );

        assert!(serde_json::from_value::<ObjectType>(
            json!({
                "description": String::from("bob"),
                "additionalProperties": {
                    "$ref": String::from("#/definitions/io.k8s.apimachinery.pkg.api.resource.Quantity"),
                },
            })
        ).is_ok());

        // Test case from failing PodSpec
        serde_json::from_value::<Type>(json!({
            "type": "object",
            "required": ["containers"],
            "properties": {
                "resourceClaims": {
                    "type": "array",
                    "items": {
                        "$ref": "#/definitions/io.k8s.api.core.v1.PodResourceClaim"
                    },
                    "x-kubernetes-list-map-keys": ["name"],
                    "x-kubernetes-list-type": "map",
                    "x-kubernetes-patch-merge-key": "name",
                    "x-kubernetes-patch-strategy": "merge,retainKeys"
                },
                "schedulingGates": {
                    "type": "array",
                    "items": {
                        "$ref": "#/definitions/io.k8s.api.core.v1.PodSchedulingGate"
                    },
                    "x-kubernetes-list-map-keys": ["name"],
                    "x-kubernetes-list-type": "map",
                    "x-kubernetes-patch-merge-key": "name",
                    "x-kubernetes-patch-strategy": "merge"
                },
                "topologySpreadConstraints": {
                    "type": "array",
                    "items": {
                        "$ref": "#/definitions/io.k8s.api.core.v1.TopologySpreadConstraint"
                    },
                    "x-kubernetes-list-map-keys": ["topologyKey", "whenUnsatisfiable"],
                    "x-kubernetes-list-type": "map",
                    "x-kubernetes-patch-merge-key": "topologyKey",
                    "x-kubernetes-patch-strategy": "merge"
                },
                "volumes": {
                    "type": "array",
                    "items": {
                        "$ref": "#/definitions/io.k8s.api.core.v1.Volume"
                    },
                    "x-kubernetes-list-map-keys": ["name"],
                    "x-kubernetes-list-type": "map",
                    "x-kubernetes-patch-merge-key": "name",
                    "x-kubernetes-patch-strategy": "merge,retainKeys"
                }
            }
        }))
        .unwrap();

        serde_json::from_value::<Type>(json!({
          "type": "object",
          "properties": {
            "minimum": {
              "type": "number",
              "format": "double"
            },
          }
        }))?;

        Ok(())
    }

    #[test]
    fn minimize_openapi_v2_spec() -> anyhow::Result<()> {
        let reader = flate2::read::GzDecoder::new(
            include_bytes!("./openapi-v2-kubernetes-1.32.json.gz").as_ref(),
        );

        let spec: Spec = serde_json::from_reader(reader)?;

        let file = File::create("src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json.tmp")?;

        // Pretty file is for diffing
        let pretty_file =
            File::create("src/patch/openapi/openapi-v2-kubernetes-1.32-minimized-pretty.json")?;
        serde_json::to_writer(file, &spec)?;

        serde_json::to_writer_pretty(pretty_file, &spec)?;

        std::fs::rename(
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json.tmp",
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized.json",
        )?;

        // Ensure the spec can be loaded
        let loaded_spec = super::Spec::load();
        let a = "/tmp/alice.json";
        let b = "/tmp/bob.json";
        if loaded_spec != &spec {
            serde_json::to_writer_pretty(File::create(a)?, &spec)?;
            serde_json::to_writer_pretty(File::create(b)?, loaded_spec)?;
            panic!(
                "loaded spec does not match written spec, wrote to {a} and {b}\nrun `diff {a} {b}` or `dyff between {a} {b}` to see the difference"
            );
        }

        Ok(())
    }

    #[test]
    fn check_openapi_spec() {
        super::Spec::load();
    }
}
