use crate::{fieldspec, manifest::Str, patch::openapi, resmap::ResourceMap};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

#[async_trait::async_trait]
impl Transformer for NamespaceTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let spec = openapi::v2::Spec::load();
        let builtin = &fieldspec::Builtin::load();
        let ns = self.0.to_string();
        for resource in resources.iter_mut() {
            if spec.is_namespaced(resource.gvk()) {
                builtin.namespace.apply::<String>(resource, |ns_ref| {
                    *ns_ref = ns.clone();
                    Ok(())
                })?;
            }

            // Quirk of kustomize is to apply namespaces to (cluster) rolebinding subjects too

            builtin.subjects.apply::<String>(resource, |ns_ref| {
                *ns_ref = ns.clone();
                Ok(())
            })?;
        }

        Ok(())
    }
}
