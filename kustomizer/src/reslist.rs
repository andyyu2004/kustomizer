use serde::{Deserialize, Serialize};

use crate::{
    manifest::{Str, TypeMeta, apiversion, kind},
    resmap::ResourceMap,
    resource::Resource,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceList {
    #[serde(flatten)]
    type_meta: TypeMeta<apiversion::ConfigV1, kind::ResourceList>,
    items: Box<[Resource]>,
}

impl From<ResourceMap> for ResourceList {
    fn from(map: ResourceMap) -> Self {
        Self::new(map)
    }
}

impl ResourceList {
    pub fn new(resources: impl IntoIterator<Item = Resource>) -> Self {
        Self {
            type_meta: TypeMeta::default(),
            items: resources.into_iter().collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Resource> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl IntoIterator for ResourceList {
    type Item = Resource;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a ResourceList {
    type Item = &'a Resource;
    type IntoIter = std::slice::Iter<'a, Resource>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
