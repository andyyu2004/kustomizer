use core::fmt;

use indexmap::IndexMap;

use crate::resource::{ResId, Resource};

#[derive(Clone, Default)]
pub struct ResourceMap {
    resources: IndexMap<ResId, Resource>,
}

impl fmt::Debug for ResourceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.resources.values()).finish()
    }
}

impl ResourceMap {
    pub fn new() -> Self {
        ResourceMap {
            resources: IndexMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn insert(&mut self, resource: Resource) -> Option<Resource> {
        self.resources.insert(resource.id.clone(), resource)
    }

    pub fn get(&self, id: &ResId) -> Option<&Resource> {
        self.resources.get(id)
    }

    pub fn remove(&mut self, id: &ResId) -> Option<Resource> {
        self.resources.shift_remove(id)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Resource> + DoubleEndedIterator {
        self.resources.values()
    }
}
