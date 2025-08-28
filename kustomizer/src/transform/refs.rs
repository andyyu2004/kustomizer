use crate::{manifest::Str, resource::RefSpecs};

#[derive(Debug)]
pub struct Rename {
    pub kind: Str,
    pub from: Str,
    pub to: Str,
}

pub struct RefsTransformer {
    ref_specs: RefSpecs,
    renames: Vec<Rename>,
}
