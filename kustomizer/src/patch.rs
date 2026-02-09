use std::{collections::HashSet, slice, str::FromStr};

use json::{Value, map::Entry};

use crate::resource::{Object, Resource};

use self::openapi::v2::{
    ArrayType, InlineOrRef, ObjectType, Spec,
    Type::{self},
};

pub mod openapi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatchStrategy {
    Delete,
    Merge,
    Replace,
    RetainKeys,
    #[serde(rename = "merge,retainKeys", alias = "retainKeys,merge")]
    MergeRetainKeys,
}

impl FromStr for PatchStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        json::from_value(Value::String(s.to_string()))
            .map_err(|err| anyhow::anyhow!("invalid patch strategy '{s}': {err}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ListType {
    Atomic,
    Set,
    Map,
}

type ShouldRetain = bool;

#[tracing::instrument(skip_all, fields(resource = %base.id()))]
pub fn merge_patch(base: &mut Resource, patch: Resource) -> anyhow::Result<ShouldRetain> {
    let spec = Spec::load_global_default();
    let schema = spec.schema_for(base.gvk());
    let (_patch_id, mut patch_root) = patch.into_parts();

    let metadata = patch_root["metadata"].as_object_mut().unwrap();
    // Don't want to overwrite the name and namespace of the base resource using the patch.
    metadata.remove("name");
    metadata.remove("namespace");
    merge_obj(base.root_mut(), patch_root, spec, schema)
}

fn merge_obj(
    base: &mut Object,
    mut patch: Object,
    spec: &Spec,
    schema: Option<&ObjectType>,
) -> anyhow::Result<ShouldRetain> {
    let patch_strategy = if let Some(directive) = patch.get("$patch").and_then(|v| v.as_str()) {
        let strategy = directive.parse::<PatchStrategy>()?;
        patch.remove("$patch");
        Some(strategy)
    } else {
        None
    };

    match patch_strategy {
        Some(PatchStrategy::Delete) => {
            base.clear();
            return Ok(false);
        }
        Some(PatchStrategy::Replace) => {
            *base = patch;
            return Ok(true);
        }
        _ => {
            for (key, value) in patch {
                if value.is_null() {
                    base.remove(&key);
                    continue;
                }

                match base.entry(key) {
                    Entry::Vacant(entry) => drop(entry.insert(value)),
                    Entry::Occupied(mut entry) => {
                        let subschema = schema.and_then(|s| s.properties.get(entry.key()));
                        if !merge(spec, entry.get_mut(), value, subschema)? {
                            entry.remove();
                        }
                    }
                }
            }
        }
    }

    Ok(true)
}

// two values match if they have at least one common element and
// corresponding elements only differ if one is an empty string
fn array_keys_match<'a>(
    keys: impl IntoIterator<Item = &'a str>,
    base: &Value,
    patch: &Value,
) -> bool {
    let mut one_match = false;
    for key in keys {
        let base_value = base.get(key);
        let patch_value = patch.get(key);
        if base_value.is_some() && patch_value.is_some() {
            if base_value == patch_value {
                one_match = true;
            } else if base_value.and_then(|v| v.as_str()) == Some("")
                || patch_value.and_then(|v| v.as_str()) == Some("")
            {
                continue;
            } else {
                return false;
            }
        }
    }

    one_match
}

fn merge_array(
    bases: &mut Vec<Value>,
    patches: Vec<Value>,
    spec: &Spec,
    schema: Option<&ArrayType>,
) -> anyhow::Result<bool> {
    let strategy_of = |patch: &Value| {
        patch
            .as_object()
            .and_then(|o| o.get("$patch"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<PatchStrategy>().ok())
    };

    let is_delete_patch = |patch: &Value| strategy_of(patch) == Some(PatchStrategy::Delete);

    let is_non_delete_patch = |patch: &Value| !is_delete_patch(patch);

    let cleaned = |mut patch: Value| {
        if let Some(o) = patch.as_object_mut() {
            o.remove("$patch");
            if o.is_empty() {
                return None;
            }
        }

        Some(patch)
    };

    let mk_non_delete_patches = |patches: Vec<Value>| {
        patches
            .into_iter()
            .filter(is_non_delete_patch)
            .filter_map(cleaned)
    };

    let delete_all = patches
        .iter()
        .any(|p| strategy_of(p) == Some(PatchStrategy::Delete) && cleaned(p.clone()).is_none());

    if delete_all {
        bases.clear();
        return Ok(false);
    }

    let force_replace = patches
        .iter()
        .any(|p| strategy_of(p) == Some(PatchStrategy::Replace));

    if force_replace {
        *bases = mk_non_delete_patches(patches).collect();
        return Ok(true);
    }

    match schema {
        Some(schema) => match schema
            .list_map_keys
            .as_deref()
            .or(schema.patch_merge_key.as_ref().map(slice::from_ref))
        {
            Some(keys) => {
                for patch in patches {
                    if let Some(pos) = bases.iter().position(|base| {
                        array_keys_match(keys.iter().map(|s| s.as_str()), base, &patch)
                    }) {
                        if !merge(spec, &mut bases[pos], patch, Some(&schema.items))? {
                            bases.remove(pos);
                        }
                    } else if is_non_delete_patch(&patch)
                        && let Some(patch) = cleaned(patch)
                    {
                        bases.push(patch);
                    }
                }
            }
            None => match schema.patch_strategy {
                Some(strategy) => match strategy {
                    PatchStrategy::Merge
                        if schema.list_type.is_none_or(|t| t != ListType::Atomic) =>
                    {
                        bases.extend(mk_non_delete_patches(patches));
                        match schema.list_type {
                            Some(ListType::Atomic) => unreachable!(),
                            Some(ListType::Set) => {
                                // For set-type lists, deduplicate and sort to match kustomize behavior
                                let unique = bases.drain(..).collect::<HashSet<_>>();
                                *bases = unique.into_iter().collect::<Vec<_>>();
                                bases.sort_by(|a, b| {
                                    // Compare values by their string representation
                                    // This handles primitive values (strings, numbers, bools)
                                    a.to_string().cmp(&b.to_string())
                                });
                            }
                            Some(ListType::Map) | None => {}
                        }

                        return Ok(true);
                    }
                    PatchStrategy::Merge | PatchStrategy::Replace => {
                        *bases = mk_non_delete_patches(patches).collect();
                    }
                    PatchStrategy::RetainKeys => todo!("array patch strategy retainKeys"),
                    PatchStrategy::MergeRetainKeys => {
                        todo!("array patch strategy merge,retainKeys")
                    }
                    PatchStrategy::Delete => todo!("array patch strategy delete"),
                },
                None => *bases = mk_non_delete_patches(patches).collect(),
            },
        },
        _ => *bases = mk_non_delete_patches(patches).collect(),
    }

    Ok(true)
}

fn merge(
    spec: &Spec,
    base: &mut Value,
    patch: Value,
    schema: Option<&InlineOrRef<Box<Type>>>,
) -> anyhow::Result<ShouldRetain> {
    let schema = schema.map(|s| spec.resolve(s));
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => match schema {
            Some(Type::Object(schema)) => merge_obj(base, patch, spec, Some(schema)),
            _ => merge_obj(base, patch, spec, None),
        },
        (Value::Array(base), Value::Array(patch)) => match schema {
            Some(Type::Array(schema)) => merge_array(base, patch, spec, Some(schema)),
            _ => merge_array(base, patch, spec, None),
        },
        (base, patch) => {
            *base = patch;
            Ok(true)
        }
    }
}
