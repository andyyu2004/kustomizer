use crate::{fieldspec, manifest::Str, patch::openapi, resmap::ResourceMap};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

#[async_trait::async_trait]
impl Transformer for NamespaceTransformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let spec = openapi::v2::Spec::load();
        let field_specs = &fieldspec::Builtin::load().namespace;
        let ns = serde_json::Value::String(self.0.to_string());
        for resource in resources.iter_mut() {
            let id = resource.id().clone();
            field_specs.apply(resource, |namespace_ref| match namespace_ref.as_str() {
                Some(_) => {
                    *namespace_ref = ns.clone();
                    Ok(())
                }
                None => anyhow::bail!("expected namespace to be a string for resource `{id}`",),
            })?;

            // if spec.is_namespaced(resource.gvk()) {
            //     resource.metadata_mut().set("namespace", self.0.clone());
            // } else {
            //     panic!("{:?} is not a namespaced resource", resource.gvk());
            // }
        }

        Ok(())
    }
}
