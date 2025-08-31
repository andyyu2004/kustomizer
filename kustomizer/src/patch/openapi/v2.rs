use std::{collections::HashSet, fmt, str::FromStr, sync::OnceLock};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    manifest::Str,
    patch::{ListType, PatchStrategy},
    resource::Gvk,
};

const SPEC_V2_GZ: &[u8] = include_bytes!("./openapi-v2-kubernetes-1.32-minimized.json");

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    definitions: IndexMap<TypeMeta, Type>,
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

    pub(crate) fn resolve<'a>(&'a self, schema: &'a InlineOrRef<Type>) -> &'a Type {
        match schema {
            InlineOrRef::Inline(ty) => ty,
            InlineOrRef::Ref(r) => {
                let def_id = TypeMeta::from_str(&r.reference["#/definitions/".len()..])
                    .expect("spec should have valid $ref");
                let schema = self
                    .definitions
                    .get(&def_id)
                    .expect("spec should have definition for $ref");
                dbg!((&def_id, &schema));
                schema
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    // For some reason, some schema definitions don't parse as a type and have no `kind` field.
    #[serde(flatten)]
    pub ty: Option<Type>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlineOrRef<T> {
    Ref(Ref),
    Inline(T),
}

// TODO custom deserializer to drop the `#/definitions/` prefix and also parse it into a TypeMeta
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Ref {
    #[serde(rename = "$ref")]
    reference: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectType {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, InlineOrRef<Type>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<InlineOrRef<Type>>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ArrayType {
    pub items: InlineOrRef<Box<Type>>,

    #[serde(
        rename = "x-kubernetes-patch-strategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub patch_strategy: Option<PatchStrategy>,

    #[serde(
        rename = "x-kubernetes-patch-key",
        skip_serializing_if = "Option::is_none"
    )]
    pub patch_key: Option<Str>,

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
    Boolean,
    Any,
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "camelCase")]
        enum TypeHelper {
            Object(ObjectType),
            Array(ArrayType),
            String,
            Integer,
            Boolean,
        }

        Ok(match TypeHelper::deserialize(deserializer) {
            Ok(TypeHelper::Object(o)) => Type::Object(o),
            Ok(TypeHelper::Array(a)) => Type::Array(a),
            Ok(TypeHelper::String) => Type::String,
            Ok(TypeHelper::Integer) => Type::Integer,
            Ok(TypeHelper::Boolean) => Type::Boolean,
            // Some weird cases we don't care about, treat as Any
            Err(_) => Type::Any,
        })
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use crate::patch::openapi::v2::InlineOrRef;

    use super::{Spec, Type};

    #[test]
    fn test_serde() -> anyhow::Result<()> {
        assert_eq!(serde_json::from_value::<Type>(json!({}))?, Type::Any);

        let ty = serde_json::from_value::<Type>(json!({
          "description": "ObjectMeta is metadata that all persisted resources must have, which includes all objects users must create.",
          "type": "object",
          "properties": {
            "annotations": {
              "description": "Annotations is an unstructured key value map stored with a resource that may be set by external tools to store and retrieve arbitrary metadata. They are not queryable and should be preserved when modifying objects. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/annotations",
              "type": "object",
              "additionalProperties": {
                "type": "string"
              }
            },
            "creationTimestamp": {
              "description": "CreationTimestamp is a timestamp representing the server time when this object was created. It is not guaranteed to be set in happens-before order across separate operations. Clients may not set this value. It is represented in RFC3339 form and is in UTC.\n\nPopulated by the system. Read-only. Null for lists. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata",
              "$ref": "#/definitions/io.k8s.apimachinery.pkg.apis.meta.v1.Time"
            },
            "deletionGracePeriodSeconds": {
              "description": "Number of seconds allowed for this object to gracefully terminate before it will be removed from the system. Only set when deletionTimestamp is also set. May only be shortened. Read-only.",
              "type": "integer",
              "format": "int64"
            },
            "deletionTimestamp": {
              "description": "DeletionTimestamp is RFC 3339 date and time at which this resource will be deleted. This field is set by the server when a graceful deletion is requested by the user, and is not directly settable by a client. The resource is expected to be deleted (no longer visible from resource lists, and not reachable by name) after the time in this field, once the finalizers list is empty. As long as the finalizers list contains items, deletion is blocked. Once the deletionTimestamp is set, this value may not be unset or be set further into the future, although it may be shortened or the resource may be deleted prior to this time. For example, a user may request that a pod is deleted in 30 seconds. The Kubelet will react by sending a graceful termination signal to the containers in the pod. After that 30 seconds, the Kubelet will send a hard termination signal (SIGKILL) to the container and after cleanup, remove the pod from the API. In the presence of network partitions, this object may still exist after this timestamp, until an administrator or automated process can determine the resource is fully terminated. If not set, graceful deletion of the object has not been requested.\n\nPopulated by the system when a graceful deletion is requested. Read-only. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata",
              "$ref": "#/definitions/io.k8s.apimachinery.pkg.apis.meta.v1.Time"
            },
            "finalizers": {
              "description": "Must be empty before the object is deleted from the registry. Each entry is an identifier for the responsible component that will remove the entry from the list. If the deletionTimestamp of the object is non-nil, entries in this list can only be removed. Finalizers may be processed and removed in any order.  Order is NOT enforced because it introduces significant risk of stuck finalizers. finalizers is a shared field, any actor with permission can reorder it. If the finalizer list is processed in order, then this can lead to a situation in which the component responsible for the first finalizer in the list is waiting for a signal (field value, external system, or other) produced by a component responsible for a finalizer later in the list, resulting in a deadlock. Without enforced ordering finalizers are free to order amongst themselves and are not vulnerable to ordering changes in the list.",
              "type": "array",
              "items": {
                "type": "string"
              },
              "x-kubernetes-list-type": "set",
              "x-kubernetes-patch-strategy": "merge"
            },
            "generateName": {
              "description": "GenerateName is an optional prefix, used by the server, to generate a unique name ONLY IF the Name field has not been provided. If this field is used, the name returned to the client will be different than the name passed. This value will also be combined with a unique suffix. The provided value has the same validation rules as the Name field, and may be truncated by the length of the suffix required to make the value unique on the server.\n\nIf this field is specified and the generated name exists, the server will return a 409.\n\nApplied only if Name is not specified. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#idempotency",
              "type": "string"
            },
            "generation": {
              "description": "A sequence number representing a specific generation of the desired state. Populated by the system. Read-only.",
              "type": "integer",
              "format": "int64"
            },
            "labels": {
              "description": "Map of string keys and values that can be used to organize and categorize (scope and select) objects. May match selectors of replication controllers and services. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/labels",
              "type": "object",
              "additionalProperties": {
                "type": "string"
              }
            },
            "managedFields": {
              "description": "ManagedFields maps workflow-id and version to the set of fields that are managed by that workflow. This is mostly for internal housekeeping, and users typically shouldn't need to set or understand this field. A workflow can be the user's name, a controller's name, or the name of a specific apply path like \"ci-cd\". The set of fields is always in the version that the workflow used when modifying the object.",
              "type": "array",
              "items": {
                "$ref": "#/definitions/io.k8s.apimachinery.pkg.apis.meta.v1.ManagedFieldsEntry"
              },
              "x-kubernetes-list-type": "atomic"
            },
            "name": {
              "description": "Name must be unique within a namespace. Is required when creating resources, although some resources may allow a client to request the generation of an appropriate name automatically. Name is primarily intended for creation idempotence and configuration definition. Cannot be updated. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names#names",
              "type": "string"
            },
            "namespace": {
              "description": "Namespace defines the space within which each name must be unique. An empty namespace is equivalent to the \"default\" namespace, but \"default\" is the canonical representation. Not all objects are required to be scoped to a namespace - the value of this field for those objects will be empty.\n\nMust be a DNS_LABEL. Cannot be updated. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces",
              "type": "string"
            },
            "ownerReferences": {
              "description": "List of objects depended by this object. If ALL objects in the list have been deleted, this object will be garbage collected. If this object is managed by a controller, then an entry in this list will point to this controller, with the controller field set to true. There cannot be more than one managing controller.",
              "type": "array",
              "items": {
                "$ref": "#/definitions/io.k8s.apimachinery.pkg.apis.meta.v1.OwnerReference"
              },
              "x-kubernetes-list-map-keys": [
                "uid"
              ],
              "x-kubernetes-list-type": "map",
              "x-kubernetes-patch-merge-key": "uid",
              "x-kubernetes-patch-strategy": "merge"
            },
            "resourceVersion": {
              "description": "An opaque value that represents the internal version of this object that can be used by clients to determine when objects have changed. May be used for optimistic concurrency, change detection, and the watch operation on a resource or set of resources. Clients must treat these values as opaque and passed unmodified back to the server. They may only be valid for a particular resource or set of resources.\n\nPopulated by the system. Read-only. Value must be treated as opaque by clients and . More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#concurrency-control-and-consistency",
              "type": "string"
            },
            "selfLink": {
              "description": "Deprecated: selfLink is a legacy read-only field that is no longer populated by the system.",
              "type": "string"
            },
            "uid": {
              "description": "UID is the unique in time and space value for this object. It is typically generated by the server on successful creation of a resource and is not allowed to change on PUT operations.\n\nPopulated by the system. Read-only. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names#uids",
              "type": "string"
            }
          }
        }))?;
        dbg!(&ty);

        match ty {
            Type::Object(ty) => {
                for prop in ty.properties.values() {
                    assert!(
                        !matches!(prop, InlineOrRef::Inline(Type::Any)),
                        "got Any in prop: {prop:?}"
                    )
                }
            }
            _ => panic!("expected ObjectType, got: {ty:?}"),
        }
        Ok(())
    }

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

        // Pretty file is for diffing
        let pretty_file = std::fs::File::create(
            "src/patch/openapi/openapi-v2-kubernetes-1.32-minimized-pretty.json",
        )?;
        serde_json::to_writer(file, &spec)?;

        serde_json::to_writer_pretty(pretty_file, &spec)?;

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
