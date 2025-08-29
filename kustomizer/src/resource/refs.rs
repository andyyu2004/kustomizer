use std::sync::OnceLock;

use crate::fieldspec::FieldSpec;
use serde::Deserialize;

use super::{Gvk, GvkMatcher};

const REFSPECS: &str = include_str!("./refspecs.yaml");

pub struct RefSpecs {
    // naive implementation, since the list is probably small
    specs: Box<[RefSpec]>,
}

impl RefSpecs {
    pub fn load_builtin() -> &'static Self {
        static INSTANCE: OnceLock<RefSpecs> = OnceLock::new();
        INSTANCE.get_or_init(|| RefSpecs {
            specs: serde_yaml::from_str(REFSPECS).expect("valid refspecs.yaml"),
        })
    }

    pub fn referrers(&self, gvk: &Gvk) -> impl Iterator<Item = &FieldSpec> {
        self.specs
            .iter()
            .filter(move |spec| spec.referee.matches(gvk))
            .flat_map(|spec| &spec.referrers)
    }
}

/// A description of how resources of one type refer to resources of another type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RefSpec {
    /// The type of the resource that is refered by the referers.
    #[serde(flatten)]
    pub referee: GvkMatcher,
    /// The field that contains the reference to the referee.
    pub referrers: Box<[FieldSpec]>,
}

#[cfg(test)]
#[test]
fn ensure_refspecs_valid() {
    RefSpecs::load_builtin();
}
