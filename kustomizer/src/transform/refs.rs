use anyhow::anyhow;
use serde_json::Value;

use crate::{
    manifest::Str,
    resmap::ResourceMap,
    resource::{RefSpecs, ResId},
};

use super::Transformer;

#[derive(Debug)]
pub struct Rename {
    res_id: ResId,
    new_name: Str,
    new_namespace: Option<Str>,
}

impl Rename {
    pub fn new(res_id: ResId, new_namespace: Option<Str>, new_name: Str) -> Self {
        assert!(!new_name.is_empty());
        Self {
            res_id,
            new_name,
            new_namespace,
        }
    }

    pub fn new_name(res_id: ResId, new_name: Str) -> Self {
        let ns = res_id.namespace.clone();
        Self::new(res_id, ns, new_name)
    }

    pub fn new_namespace(res_id: ResId, new_namespace: Str) -> Self {
        let name = res_id.name.clone();
        Self::new(res_id, Some(new_namespace), name)
    }
}

pub struct RenameTransformer<'a> {
    ref_specs: &'a RefSpecs,
    // FIXME: What if a resource is renamed multiple times?
    renames: &'a [Rename],
}

impl<'a> RenameTransformer<'a> {
    pub fn new(ref_specs: &'a RefSpecs, renames: &'a [Rename]) -> Self {
        Self { ref_specs, renames }
    }
}

impl Transformer for RenameTransformer<'_> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        // Naive implementation with nested loops

        // For every resource that has been renamed
        for rename in self.renames {
            // For every referrer that can refer to this resource
            for referrer in self.ref_specs.referrers(&rename.res_id.gvk) {
                // Search for references in every resource
                for resource in resources.iter_mut() {
                    referrer.apply::<Value>(resource, &mut |v| {
                        match v {
                            Value::String(s) => {
                                if s == &rename.res_id.name {
                                    *s = rename.new_name.to_string();
                                }
                            }
                            Value::Object(map) => {
                                let name = map
                                    .get("name")
                                    .ok_or_else(|| anyhow!("referrer field missing `name` field"))?
                                    .as_str()
                                    .ok_or_else(|| {
                                        anyhow!("referrer field `name` field is not a string")
                                    })?;
                                let kind = map.get("kind").and_then(|k| k.as_str());
                                let namespace = map.get("namespace").and_then(|n| n.as_str());
                                let res_id = &rename.res_id;
                                if kind.is_none_or(|kind| kind == res_id.gvk.kind)
                                    && name == res_id.name
                                    && namespace == res_id.namespace.as_deref()
                                {
                                    map.insert(
                                        "name".to_string(),
                                        Value::String(rename.new_name.to_string()),
                                    );
                                    if let Some(new_ns) = &rename.new_namespace {
                                        map.insert(
                                            "namespace".to_string(),
                                            Value::String(new_ns.to_string()),
                                        );
                                    } else {
                                        map.remove("namespace");
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }

                        Ok(())
                    })?;
                }
            }
        }

        Ok(())
    }
}
