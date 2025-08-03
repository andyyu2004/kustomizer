use core::fmt;
use std::ops::{Index, IndexMut};

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
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn insert(&mut self, resource: Resource) -> Result<(), Resource> {
        if let Some(old) = self.resources.insert(resource.id.clone(), resource) {
            Err(old)?
        }

        Ok(())
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Resource> + DoubleEndedIterator {
        self.resources.values()
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = &mut Resource> + DoubleEndedIterator {
        self.resources.values_mut()
    }

    /// In-place merge of two `ResourceMap`s, any conflicting resources will be an error
    pub fn merge(&mut self, other: ResourceMap) -> Result<(), Resource> {
        for (_, resource) in other.resources {
            self.insert(resource)?;
        }
        Ok(())
    }
}

impl Index<&ResId> for ResourceMap {
    type Output = Resource;

    fn index(&self, id: &ResId) -> &Self::Output {
        self.resources
            .get(id)
            .unwrap_or_else(|| panic!("resource with id `{id}` not in ResourceMap"))
    }
}

impl IndexMut<&ResId> for ResourceMap {
    fn index_mut(&mut self, id: &ResId) -> &mut Self::Output {
        self.resources
            .get_mut(id)
            .unwrap_or_else(|| panic!("resource with id `{id}` not in ResourceMap"))
    }
}
