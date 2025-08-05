use crate::{plugin::FunctionPlugin, resmap::ResourceMap};

use super::Transformer;

#[async_trait::async_trait]
impl Transformer for FunctionPlugin {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        match self.spec() {
            crate::manifest::FunctionSpec::Exec(exec_spec) => todo!(),
            crate::manifest::FunctionSpec::Container(container_spec) => todo!(),
        }
    }
}
