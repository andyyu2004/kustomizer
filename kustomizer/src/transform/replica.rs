use std::collections::HashMap;

use crate::{
    fieldspec,
    manifest::{Replica, Str},
};

use super::Transformer;

pub struct ReplicaTransformer {
    replicas: HashMap<Str, u32>,
}

impl ReplicaTransformer {
    pub fn new(replicas: &[Replica]) -> Self {
        Self {
            replicas: replicas
                .iter()
                .map(|replica| (replica.name.clone(), replica.count))
                .collect(),
        }
    }
}

#[async_trait::async_trait]
impl Transformer for ReplicaTransformer {
    async fn transform(
        &mut self,
        resources: &mut crate::resmap::ResourceMap,
    ) -> anyhow::Result<()> {
        let field_specs = &fieldspec::Builtin::get().replicas;
        for resource in resources.iter_mut() {
            if let Some(desired_replicas) = self.replicas.get(resource.name()) {
                let id = resource.id().clone();
                field_specs.apply(resource, |replicas| {
                    if !replicas.is_number() {
                        anyhow::bail!("expected replicas to be a number for resource `{id}`",);
                    }
                    *replicas =
                        serde_yaml::Value::Number(serde_yaml::Number::from(*desired_replicas));

                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}
