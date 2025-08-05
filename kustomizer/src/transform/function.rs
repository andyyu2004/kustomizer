use crate::{plugin::FunctionPlugin, resmap::ResourceMap};

use super::Transformer;

#[async_trait::async_trait]
impl Transformer for FunctionPlugin {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        todo!()
    }
}
