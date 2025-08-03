use std::{path::Path, process::Stdio};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use anyhow::{Context, bail};

use crate::{manifest::FunctionSpec, reslist::ResourceList, resmap::ResourceMap};

#[async_trait::async_trait]
pub trait Generator {
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceMap>;
}

pub struct FunctionGenerator {
    spec: FunctionSpec,
}

impl FunctionGenerator {
    pub fn new(spec: FunctionSpec) -> Self {
        Self { spec }
    }
}

#[async_trait::async_trait]
impl Generator for FunctionGenerator {
    #[tracing::instrument(skip_all, fields(workdir = %workdir.display()))]
    async fn generate(
        &mut self,
        workdir: &Path,
        input: &ResourceList,
    ) -> anyhow::Result<ResourceMap> {
        let mut resmap = ResourceMap::default();
        match &self.spec {
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

                let resources = serde_yaml::from_slice::<ResourceList>(&output.stdout)?;
                for resource in resources {
                    if let Err(old) = resmap.insert(resource) {
                        bail!("duplicate resource `{}` found in function output", old.id);
                    }
                }
            }
            FunctionSpec::Container(_spec) => {
                bail!("Container functions are not supported yet")
            }
        }

        Ok(resmap)
    }
}
