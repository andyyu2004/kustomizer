use indexmap::IndexMap;

use crate::manifest::ResId;

pub struct ResourceMap {
    resources: IndexMap<ResId, serde_yaml::Value>,
}
