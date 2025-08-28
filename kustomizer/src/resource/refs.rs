use std::{collections::HashMap, sync::OnceLock};

use crate::fieldspec::FieldSpec;
use serde::Deserialize;

use super::GvkMatcher;

const REFSPECS: &str = include_str!("./refspecs.yaml");

pub struct RefSpecs {
    specs: HashMap<GvkMatcher, Box<[FieldSpec]>>,
}

impl RefSpecs {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceLock<RefSpecs> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let specs =
                serde_yaml::from_str::<Vec<RefSpec>>(REFSPECS).expect("valid refspecs.yaml");
            RefSpecs {
                specs: specs
                    .into_iter()
                    .map(|s| (s.referee, s.referrers))
                    .collect(),
            }
        })
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
    RefSpecs::load();
}
