mod configmap;
mod function;

pub use self::configmap::ConfigMapGenerator;
pub use self::function::FunctionGenerator;

use crate::reslist::ResourceList;
use std::path::Path;

#[async_trait::async_trait]
pub trait Generator {
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceList>;
}
