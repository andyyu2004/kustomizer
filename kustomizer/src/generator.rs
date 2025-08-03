use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, bail};

use crate::{manifest::FunctionSpec, reslist::ResourceList, resmap::ResourceMap};

pub trait Generator {
    fn generate(&mut self, workdir: &Path, input: &ResourceList) -> anyhow::Result<ResourceMap>;
}

pub struct FunctionGenerator {
    spec: FunctionSpec,
}

impl FunctionGenerator {
    pub fn new(spec: FunctionSpec) -> Self {
        Self { spec }
    }
}

impl Generator for FunctionGenerator {
    fn generate(&mut self, workdir: &Path, input: &ResourceList) -> anyhow::Result<ResourceMap> {
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

                serde_yaml::to_writer(proc.stdin.as_mut().unwrap(), input)
                    .with_context(|| "failed to write input to function command")?;

                let stdout = proc.stdout.take().unwrap();

                let output = proc.wait_with_output()?;
                if !output.status.success() {
                    bail!(
                        "function command failed with status {}: {}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                let resources = serde_yaml::from_reader::<_, ResourceList>(stdout)?;
                for resource in resources {
                    if let Some(old) = resmap.insert(resource) {
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
