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

impl Transformer for ReplicaTransformer {
    #[tracing::instrument(skip_all)]
    async fn transform(
        &mut self,
        resources: &mut crate::resmap::ResourceMap,
    ) -> anyhow::Result<()> {
        let field_specs = &fieldspec::Builtin::load().replicas;
        for resource in resources.iter_mut() {
            if let Some(desired_replicas) = self.replicas.get(resource.name()).copied() {
                field_specs.apply::<u64>(resource, |replicas| {
                    *replicas = desired_replicas as u64;
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}
