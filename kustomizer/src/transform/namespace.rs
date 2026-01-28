use crate::{
    fieldspec,
    manifest::{Str, kind},
    patch::openapi,
    resmap::ResourceMap,
    resource::Object,
};

use super::Transformer;

pub struct NamespaceTransformer(pub Str);

impl Transformer for NamespaceTransformer {
    #[tracing::instrument(skip_all, name = "namespace_transform", fields(namespace = %self.0))]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let spec = openapi::v2::Spec::load();
        let builtin = &fieldspec::Builtin::load();
        let target_namespace = self.0.to_string();

        // A fresh map is allocated because a namespace change modifies the identity of the
        // resources, which can't be done in-place.
        let mut transformed_resources = ResourceMap::with_capacity(resources.len());

        for mut resource in std::mem::take(resources) {
            // Apply defaultOnly subject transformation: only ServiceAccount subjects named "default"
            // get their namespace updated. This matches kustomize's default behavior.
            self.apply_default_subject_transformation(builtin, &mut resource, &target_namespace)?;

            // Transform the resource itself based on its type
            let transformed_resource = self.transform_resource(resource, spec)?;
            transformed_resources.insert(transformed_resource)?;
        }

        *resources = transformed_resources;
        Ok(())
    }
}

impl NamespaceTransformer {
    /// Apply namespace transformation to ServiceAccount subjects named "default"
    fn apply_default_subject_transformation(
        &self,
        builtin: &fieldspec::Builtin,
        resource: &mut crate::resource::Resource,
        target_namespace: &str,
    ) -> anyhow::Result<()> {
        builtin.subjects.apply::<Object>(resource, |subject| {
            if self.is_default_service_account_subject(subject) {
                subject.insert(
                    "namespace".to_string(),
                    serde_json::Value::String(target_namespace.to_string()),
                );
            }
            Ok(())
        })
    }

    /// Check if a subject is a ServiceAccount named "default"
    fn is_default_service_account_subject(&self, subject: &Object) -> bool {
        let kind = subject.get("kind").and_then(|k| k.as_str());
        let name = subject.get("name").and_then(|n| n.as_str());

        matches!((kind, name), (Some(k), Some("default")) if kind::ServiceAccount == *k)
    }

    /// Transform an individual resource based on its type
    fn transform_resource(
        &self,
        resource: crate::resource::Resource,
        spec: &openapi::v2::Spec,
    ) -> anyhow::Result<crate::resource::Resource> {
        if kind::Namespace == **resource.kind() {
            // For Namespace resources, update the name, not the namespace
            Ok(resource.with_name(self.0.clone()))
        } else if spec.is_namespaced(resource.gvk()) {
            // For namespaced resources, update their namespace
            Ok(resource.with_namespace(Some(self.0.clone())))
        } else {
            // Cluster-scoped resources remain unchanged
            Ok(resource)
        }
    }
}
