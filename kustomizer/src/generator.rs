mod common;
mod configmap;
mod function;
mod secret;

pub use self::configmap::ConfigMapGenerator;
pub use self::secret::SecretGenerator;

use crate::reslist::ResourceList;
use std::path::Path;

pub trait Generator {
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceList>;
}
