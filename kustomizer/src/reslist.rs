use serde::{Deserialize, Serialize};

use crate::{manifest::kind, resource::Resource};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResourceList {
    kind: kind::ResourceList,
    items: Box<[Resource]>,
}
impl ResourceList {
    pub(crate) fn new(resources: impl IntoIterator<Item = Resource>) -> Self {
        Self {
            kind: kind::ResourceList,
            items: resources.into_iter().collect(),
        }
    }
}

impl IntoIterator for ResourceList {
    type Item = Resource;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_vec().into_iter()
    }
}
