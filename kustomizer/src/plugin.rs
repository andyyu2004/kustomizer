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
}
