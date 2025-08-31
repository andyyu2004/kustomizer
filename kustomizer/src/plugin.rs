use std::{path::Path, process::Stdio, time::Instant};

use anyhow::{Context, bail};
use tokio::io::AsyncWriteExt as _;

use crate::{manifest::FunctionSpec, reslist::ResourceList};

pub struct FunctionPlugin {
    spec: FunctionSpec,
}

impl FunctionPlugin {
    pub fn new(spec: FunctionSpec) -> Self {
        Self { spec }
    }

    pub fn spec(&self) -> &FunctionSpec {
        &self.spec
    }

    pub async fn exec_krm(
        &self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceList> {
        let (mut proc, cmd) = match self.spec() {
            FunctionSpec::Exec(spec) => {
                let mut cmd = tokio::process::Command::new(&spec.path);
                cmd.args(&spec.args)
                    .envs(&spec.env)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .current_dir(workdir);
                (
                    cmd.spawn().with_context(|| {
                        format!("spawn function command at `{}`", spec.path.display())
                    })?,
                    cmd,
                )
            }
            FunctionSpec::Container(_spec) => {
                anyhow::bail!("Container functions are not supported yet")
            }
        };

        let now = Instant::now();

        let stdin = serde_yaml::to_string(input)?;
        proc.stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .await
            .context("write to function stdin")?;

        let output = proc
            .wait_with_output()
            .await
            .context("wait for function process")?;
        if !output.status.success() {
            bail!(
                "function command failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        tracing::info!(
            duration = ?now.elapsed(),
            cmd = ?cmd.as_std(),
            "executed process"
        );

        Ok(serde_yaml::from_slice::<ResourceList>(&output.stdout)?)
    }
}
