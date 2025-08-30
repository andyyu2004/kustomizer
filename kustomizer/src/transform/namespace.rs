use crate::{
    fieldspec,
    manifest::{Str, kind},
    patch::openapi,
    resmap::ResourceMap,
};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

impl Transformer for NamespaceTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let spec = openapi::v2::Spec::load();
        let builtin = &fieldspec::Builtin::load();
        let ns = self.0.to_string();
        // A fresh map is allocated because a namespace change is modifying the identity of the
        // resources, which can't be done in-place.
        let mut out = ResourceMap::with_capacity(resources.len());
        for mut resource in std::mem::take(resources) {
            // Quirk of kustomize is to apply namespaces to (cluster) rolebinding subjects too
            // Only apply to ServiceAccount subjects named "default" (defaultOnly behavior in kustomize)
            builtin
                .subjects
                .apply::<serde_json::Value>(&mut resource, |subject| {
                    if let Some(kind) = subject.get("kind").and_then(|k| k.as_str())
                        && let Some(name) = subject.get("name").and_then(|n| n.as_str())
                        && kind::ServiceAccount == *kind
                        && name == "default"
                    {
                        *subject.get_mut("namespace").unwrap() =
                            serde_json::Value::String(ns.clone());
                    }
                    Ok(())
                })?;

            if spec.is_namespaced(resource.gvk()) {
                out.insert(resource.with_namespace(self.0.clone()))?;
            } else {
                out.insert(resource)?;
            }
        }

        *resources = out;

        Ok(())
    }
}
