use serde_json::{Value, map::Entry};

use crate::resource::{Object, Resource};

use self::openapi::v2::{ObjectSchema, Schema};

pub mod openapi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PatchStrategy {
    Merge,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListType {
    Atomic,
    Set,
    Map,
}

pub fn patch(base: &mut Resource, patch: Resource) -> anyhow::Result<()> {
    let spec = openapi::v2::Spec::load();
    let schema = spec.schema_for(base.gvk());
    let (_patch_id, mut patch_root) = patch.into_parts();

    let metadata = patch_root["metadata"].as_object_mut().unwrap();
    // Don't want to overwrite the name and namespace of the base resource using the patch.
    metadata.remove("name");
    metadata.remove("namespace");
    merge_obj(base.root_mut(), patch_root, schema)?;
    Ok(())
}

fn merge_obj(
    base: &mut Object,
    patch: Object,
    schema: Option<&ObjectSchema>,
) -> anyhow::Result<()> {
    for (key, value) in patch {
        match base.entry(key) {
            Entry::Vacant(entry) => drop(entry.insert(value)),
            Entry::Occupied(entry) => {
                let subschema = schema.and_then(|s| s.properties.get(entry.key()));
                merge(entry.into_mut(), value, None)?
            }
        }
    }
    Ok(())
}

fn merge(base: &mut Value, patch: Value, schema: Option<&Schema>) -> anyhow::Result<()> {
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => merge_obj(base, patch, None)?,
        (base, patch) => *base = patch,
    }

    Ok(())
}
