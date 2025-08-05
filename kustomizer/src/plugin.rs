use std::{path::Path, process::Stdio};

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
        let mut proc = match self.spec() {
            FunctionSpec::Exec(spec) => tokio::process::Command::new(&spec.path)
                .args(&spec.args)
                .envs(&spec.env)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(workdir)
                .spawn()
                .with_context(|| format!("spawn function command at `{}`", spec.path.display()))?,
            FunctionSpec::Container(_spec) => {
                anyhow::bail!("Container functions are not supported yet")
            }
        };

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
