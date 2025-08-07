use crate::{fieldspec, manifest::Str, patch::openapi, resmap::ResourceMap};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

#[async_trait::async_trait]
impl Transformer for NamespaceTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let spec = openapi::v2::Spec::load();
        let builtin = &fieldspec::Builtin::load();
        let ns = serde_json::Value::String(self.0.to_string());
        for resource in resources.iter_mut() {
            let id = resource.id().clone();
            if spec.is_namespaced(resource.gvk()) {
                builtin
                    .namespace
                    .apply(resource, |ns_ref| match ns_ref.as_str() {
                        Some(_) => {
                            *ns_ref = ns.clone();
                            Ok(())
                        }
                        None => {
                            anyhow::bail!("expected namespace to be a string for resource `{id}`",)
                        }
                    })?;
            }

            // Quirk of kustomize is to apply namespaces to (cluster) rolebinding subjects too

            builtin.subjects.apply(resource, |ns_ref| {
                if ns_ref.is_string() {
                    *ns_ref = ns.clone();
                } else {
                    anyhow::bail!(
                        "expected subjects[].namespace to be a string for resource `{id}`"
                    );
                }
                Ok(())
            })?;
        }

        Ok(())
    }
}
