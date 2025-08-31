use crate::{manifest::Str, resmap::ResourceMap};

use super::Transformer;

pub struct NameTransformer<F> {
    f: F,
}

impl<F: FnMut(&str) -> Str> NameTransformer<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: FnMut(&str) -> Str + Send> Transformer for NameTransformer<F> {
    #[tracing::instrument(skip_all, name = "name_transform")]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        // A fresh map is allocated because changing names of resources modifies their identity,
        // which can't be done in-place.
        let mut out = ResourceMap::with_capacity(resources.len());

        for resource in std::mem::take(resources) {
            let new_name = (self.f)(resource.name());
            out.insert(resource.with_name(new_name))?;
        }

        *resources = out;
        Ok(())
    }
}
