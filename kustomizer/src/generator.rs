use crate::resmap::ResourceMap;

pub trait Generator {
    fn generate(&mut self) -> anyhow::Result<ResourceMap>;
}
