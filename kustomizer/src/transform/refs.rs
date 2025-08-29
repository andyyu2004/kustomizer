use crate::{
    manifest::Str,
    resmap::ResourceMap,
    resource::{RefSpecs, ResId},
};

use super::Transformer;

#[derive(Debug)]
pub struct Rename {
    res_id: ResId,
    new_name: Str,
}

impl Rename {
    pub fn new(res_id: ResId, new_name: Str) -> Self {
        assert!(!new_name.is_empty());
        assert_ne!(res_id.name, new_name, "rename must change the name");
        Self { res_id, new_name }
    }
}

pub struct RenameTransformer<'a> {
    ref_specs: &'a RefSpecs,
    // FIXME: What if a resource is renamed multiple times?
    renames: &'a [Rename],
}

impl<'a> RenameTransformer<'a> {
    pub fn new(ref_specs: &'a RefSpecs, renames: &'a [Rename]) -> Self {
        Self { ref_specs, renames }
    }
}

impl Transformer for RenameTransformer<'_> {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        // Naive implementation with nested loops

        // For every resource that has been renamed
        for rename in self.renames {
            // For every referrer that can refer to this resource
            for referrer in self.ref_specs.referrers(&rename.res_id.gvk) {
                todo!("{:?} {:?}", rename, referrer);
            }
        }

        Ok(())
    }
}
