use anyhow::anyhow;
use serde_json::Value;

use crate::{
    manifest::Str,
    resmap::ResourceMap,
    resource::{RefSpecs, ResId},
};

use super::Transformer;

/// Represents a resource rename operation (name and/or namespace change)
#[derive(Debug)]
pub struct Rename {
    /// The original resource identifier
    res_id: ResId,
    /// The new name for the resource
    new_name: Str,
    /// The new namespace for the resource (None means remove namespace)
    new_namespace: Option<Str>,
}

impl Rename {
    /// Create a new rename operation with both name and namespace changes
    pub fn new(res_id: ResId, new_namespace: Option<Str>, new_name: Str) -> Self {
        assert!(!new_name.is_empty(), "new_name cannot be empty");
        Self {
            res_id,
            new_name,
            new_namespace,
        }
    }

    /// Create a rename operation that only changes the name
    pub fn new_name(res_id: ResId, new_name: Str) -> Self {
        let namespace = res_id.namespace.clone();
        Self::new(res_id, namespace, new_name)
    }

    /// Create a rename operation that only changes the namespace
    pub fn new_namespace(res_id: ResId, new_namespace: Str) -> Self {
        let name = res_id.name.clone();
        Self::new(res_id, Some(new_namespace), name)
    }
}

/// Transformer that updates references when resources are renamed or moved to different namespaces
pub struct RenameTransformer<'a> {
    ref_specs: &'a RefSpecs,
    renames: &'a [Rename],
}

impl<'a> RenameTransformer<'a> {
    pub fn new(ref_specs: &'a RefSpecs, renames: &'a [Rename]) -> Self {
        Self { ref_specs, renames }
    }
}

impl Transformer for RenameTransformer<'_> {
    #[tracing::instrument(skip_all)]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        for rename in self.renames {
            self.apply_rename_to_references(rename, resources)?;
        }
        Ok(())
    }
}

impl RenameTransformer<'_> {
    /// Apply a single rename operation to all references in the resource map
    fn apply_rename_to_references(
        &self,
        rename: &Rename,
        resources: &mut ResourceMap,
    ) -> anyhow::Result<()> {
        // For every referrer that can refer to this resource type
        for referrer_spec in self.ref_specs.referrers(&rename.res_id.gvk) {
            // Update references in every resource
            for resource in resources.iter_mut() {
                referrer_spec.apply::<Value>(resource, &mut |reference_value| {
                    self.update_reference_if_matches(reference_value, rename)
                })?;
            }
        }
        Ok(())
    }

    /// Update a reference value if it matches the resource being renamed
    fn update_reference_if_matches(
        &self,
        reference_value: &mut Value,
        rename: &Rename,
    ) -> anyhow::Result<()> {
        match reference_value {
            Value::String(name) => {
                // Simple name-only reference
                if name == &rename.res_id.name {
                    *name = rename.new_name.to_string();
                }
            }
            Value::Object(reference_map) => {
                // Complex reference with name, kind, and possibly namespace
                if self.reference_matches_resource(reference_map, &rename.res_id)? {
                    self.update_reference_fields(reference_map, rename);
                }
            }
            _ => {
                return Err(anyhow!(
                    "Unexpected reference value type: expected string or object"
                ));
            }
        }
        Ok(())
    }

    /// Check if a reference object matches the resource being renamed
    fn reference_matches_resource(
        &self,
        reference_map: &serde_json::Map<String, Value>,
        res_id: &ResId,
    ) -> anyhow::Result<bool> {
        let name = reference_map
            .get("name")
            .ok_or_else(|| anyhow!("Reference object missing 'name' field"))?
            .as_str()
            .ok_or_else(|| anyhow!("Reference 'name' field is not a string"))?;

        let kind = reference_map.get("kind").and_then(|k| k.as_str());
        let namespace = reference_map.get("namespace").and_then(|n| n.as_str());

        Ok(name == res_id.name
            && kind.is_none_or(|k| k == res_id.gvk.kind)
            && namespace == res_id.namespace.as_deref())
    }

    /// Update the name and namespace fields in a reference object
    fn update_reference_fields(
        &self,
        reference_map: &mut serde_json::Map<String, Value>,
        rename: &Rename,
    ) {
        // Update the name
        reference_map.insert(
            "name".to_string(),
            Value::String(rename.new_name.to_string()),
        );

        // Update or remove the namespace
        match &rename.new_namespace {
            Some(new_namespace) => {
                reference_map.insert(
                    "namespace".to_string(),
                    Value::String(new_namespace.to_string()),
                );
            }
            None => {
                reference_map.remove("namespace");
            }
        }
    }
}
