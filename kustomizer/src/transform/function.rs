use std::path::Path;

use crate::{plugin::FunctionPlugin, reslist::ResourceList, resmap::ResourceMap};

use super::Transformer;

impl Transformer for FunctionPlugin {
    async fn transform(&mut self, input: &mut ResourceMap) -> anyhow::Result<()> {
        let resources = ResourceList::from(std::mem::take(input));
        let output = self.exec_krm(Path::new("."), &resources).await?;
        input.extend(output)?;
        Ok(())
    }
}
