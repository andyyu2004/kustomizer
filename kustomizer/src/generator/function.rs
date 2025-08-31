use super::Generator;
use crate::plugin::FunctionPlugin;
use crate::reslist::ResourceList;
use std::path::Path;

impl Generator for FunctionPlugin {
    #[tracing::instrument(skip_all, name = "generate_function", fields(workdir = %workdir.display()))]
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceList> {
        self.exec_krm(workdir, input).await
    }
}
