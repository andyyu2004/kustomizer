use serde_json::{Value, map::Entry};

use crate::resource::{AnyObject, Resource};

use self::openapi::v2::ObjectSchema;

pub mod openapi;

pub fn patch(base: &mut Resource, patch: Resource) -> anyhow::Result<()> {
    let spec = openapi::v2::Spec::load();
    let schema = spec.schema_for(base.gvk());
    let (_, mut patch_root) = patch.into_parts();
    // HACK: don't want to patch the name field.
    patch_root["metadata"]
        .as_object_mut()
        .unwrap()
        .remove("name");
    merge_obj(base.root_mut(), patch_root, schema)?;
    Ok(())
}

fn merge_obj(
    base: &mut AnyObject,
    patch: AnyObject,
    _schema: Option<&ObjectSchema>,
) -> anyhow::Result<()> {
    for (key, value) in patch {
        match base.entry(key) {
            Entry::Vacant(entry) => drop(entry.insert(value)),
            Entry::Occupied(entry) => merge(entry.into_mut(), value)?,
        }
    }
    Ok(())
}

fn merge(base: &mut Value, patch: Value) -> anyhow::Result<()> {
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => merge_obj(base, patch, None)?,
        (base, patch) => *base = patch,
    }

    Ok(())
}
