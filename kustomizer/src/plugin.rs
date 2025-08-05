use std::{io, process::Stdio};

use anyhow::Context;

use crate::manifest::FunctionSpec;

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

    pub fn spawn(&self) -> anyhow::Result<tokio::process::Child> {
        let child = match self.spec() {
            FunctionSpec::Exec(spec) => tokio::process::Command::new(&spec.path)
                .args(&spec.args)
                .envs(&spec.env)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("spawn function command at `{}`", spec.path.display()))?,
            FunctionSpec::Container(_spec) => {
                anyhow::bail!("Container functions are not supported yet")
            }
        };
        Ok(child)
    }
}
