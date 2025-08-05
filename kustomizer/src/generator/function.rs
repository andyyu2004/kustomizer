use super::Generator;
use crate::plugin::FunctionPlugin;
use crate::{manifest::FunctionSpec, reslist::ResourceList};
use anyhow::{Context, bail};
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[async_trait::async_trait]
impl Generator for FunctionPlugin {
    #[tracing::instrument(skip_all, fields(workdir = %workdir.display()))]
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceList> {
        match self.spec() {
            FunctionSpec::Exec(spec) => {
                let mut proc = Command::new(&spec.path)
                    .args(&spec.args)
                    .envs(&spec.env)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .current_dir(workdir)
                    .spawn()
                    .with_context(|| {
                        format!("failed to spawn command at {}", spec.path.display())
                    })?;

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
            FunctionSpec::Container(_spec) => {
                bail!("Container functions are not supported yet")
            }
        }
    }
}
