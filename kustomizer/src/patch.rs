use serde_json::{Value, map::Entry};

use crate::resource::{Object, Resource};

use self::openapi::v2::{
    ArrayType, InlineOrRef, ObjectType, Spec,
    Type::{self},
};

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
    let spec = Spec::load();
    let schema = spec.schema_for(base.gvk());
    let (_patch_id, mut patch_root) = patch.into_parts();

    let metadata = patch_root["metadata"].as_object_mut().unwrap();
    // Don't want to overwrite the name and namespace of the base resource using the patch.
    metadata.remove("name");
    metadata.remove("namespace");
    merge_obj(base.root_mut(), patch_root, schema)?;
    Ok(())
}

fn merge_obj(base: &mut Object, patch: Object, schema: Option<&ObjectType>) -> anyhow::Result<()> {
    for (key, value) in patch {
        match base.entry(key) {
            Entry::Vacant(entry) => drop(entry.insert(value)),
            Entry::Occupied(entry) => {
                let subschema = schema.and_then(|s| s.properties.get(entry.key()));
                merge(entry.into_mut(), value, subschema)?
            }
        }
    }
    Ok(())
}

fn merge_array(
    base: &mut Vec<Value>,
    patch: Vec<Value>,
    schema: Option<&ArrayType>,
) -> anyhow::Result<()> {
    match schema {
        Some(schema) => {
            if matches!(schema.patch_strategy, Some(PatchStrategy::Merge)) {
                panic!("Replace strategy not implemented yet");
                base.extend(patch)
            } else {
                *base = patch
            }
        }
        _ => *base = patch,
    }
    Ok(())
}

fn merge(base: &mut Value, patch: Value, schema: Option<&InlineOrRef<Type>>) -> anyhow::Result<()> {
    let schema = schema.map(|s| Spec::load().resolve(s));
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => match schema {
            Some(Type::Object(schema)) => merge_obj(base, patch, Some(schema))?,
            _ => merge_obj(base, patch, None)?,
        },
        (Value::Array(base), Value::Array(patch)) => match schema {
            Some(Type::Array(schema)) => merge_array(base, patch, Some(schema))?,
            _ => merge_array(base, patch, None)?,
        },
        (base, patch) => *base = patch,
    }

    Ok(())
}
