use std::process::{Command, Stdio};

use anyhow::{Context, bail};

use crate::{manifest::FunctionSpec, resmap::ResourceMap, resource::Resource};

pub trait Generator {
    fn generate(&mut self, input: &Resource) -> anyhow::Result<ResourceMap>;
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
    fn generate(&mut self, input: &Resource) -> anyhow::Result<ResourceMap> {
        let mut resources = ResourceMap::default();
        match &self.spec {
            FunctionSpec::Exec(spec) => {
                let mut proc = Command::new(&spec.path)
                    .args(&spec.args)
                    .envs(&spec.env)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .with_context(|| {
                        format!("failed to spawn command at {}", spec.path.display())
                    })?;

                serde_yaml::to_writer(proc.stdin.as_mut().unwrap(), input)
                    .with_context(|| "failed to write input to function command")?;

                let output = proc.wait_with_output()?;
                dbg!(output);
            }
            FunctionSpec::Container(_spec) => {
                bail!("Container functions are not supported yet")
            }
        }

        Ok(resources)
    }
}
