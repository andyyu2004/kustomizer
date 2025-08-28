use crate::{manifest::Str, resmap::ResourceMap, resource::RefSpecs};

use super::Transformer;

#[derive(Debug)]
pub struct Rename {
    pub kind: Str,
    pub from: Str,
    pub to: Str,
}

pub struct RefsTransformer<'a> {
    ref_specs: &'a RefSpecs,
    renames: &'a [Rename],
}

impl<'a> RefsTransformer<'a> {
    pub fn new(ref_specs: &'a RefSpecs, renames: &'a [Rename]) -> Self {
        Self { ref_specs, renames }
    }
}

impl Transformer for RefsTransformer<'_> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        todo!()
    }
}
