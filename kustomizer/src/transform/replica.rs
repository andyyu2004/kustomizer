use std::collections::HashMap;

use crate::{
    fieldspec,
    manifest::{Replica, Str},
    resmap::ResourceMap,
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
    #[tracing::instrument(skip_all, name = "replica_transform")]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let field_specs = &fieldspec::Builtin::load().replicas;
        for (name, replicas) in &self.replicas {
            for resource in resources.iter_mut() {
                if resource.any_id_matches(|id| id.name == name) {
                    field_specs.apply::<u64>(resource, |replicas_field| {
                        *replicas_field = *replicas as u64;
                        Ok(())
                    })?;
                }
            }
        }

        Ok(())
    }
}
