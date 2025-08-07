use crate::{fieldspec, resmap::ResourceMap};

use super::Transformer;

pub struct NameTransformer<F> {
    f: F,
}

impl<F: FnMut(&str) -> String> NameTransformer<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

#[async_trait::async_trait]
impl<F: FnMut(&str) -> String + Send> Transformer for NameTransformer<F> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        let field_specs = &fieldspec::Builtin::load().name;
        // FIXME need to re-insert into the resource map and rebuild the resource as a name change is an identity change.
        for resource in resources.iter_mut() {
            let id = resource.id().clone();
            field_specs.apply::<String>(resource, |name_ref| {
                *name_ref = (self.f)(id.name.as_str());
                Ok(())
            })?;
        }
        Ok(())
    }
}
