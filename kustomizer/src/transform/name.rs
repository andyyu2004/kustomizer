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
        for resource in resources.iter_mut() {
            let id = resource.id().clone();
            field_specs.apply(resource, |name_ref| match name_ref.as_str() {
                Some(name) => {
                    *name_ref = serde_json::Value::String((self.f)(name));
                    Ok(())
                }
                None => anyhow::bail!("expected name to be a string for resource `{id}`"),
            })?;
        }
        Ok(())
    }
}
