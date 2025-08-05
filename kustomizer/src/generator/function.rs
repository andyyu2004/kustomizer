use super::Generator;
use crate::plugin::FunctionPlugin;
use crate::reslist::ResourceList;
use anyhow::bail;
use std::path::Path;
use tokio::io::AsyncWriteExt;

#[async_trait::async_trait]
impl Generator for FunctionPlugin {
    #[tracing::instrument(skip_all, fields(workdir = %workdir.display()))]
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceList> {
        let mut proc = self.spawn()?;

        let stdin = serde_yaml::to_string(input)?;
        proc.stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .await?;

        let output = proc.wait_with_output().await?;
        if !output.status.success() {
            bail!(
                "function command failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(serde_yaml::from_slice::<ResourceList>(&output.stdout)?)
    }
}
