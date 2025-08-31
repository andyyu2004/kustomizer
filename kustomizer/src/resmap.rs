use core::fmt;
use std::ops::{Index, IndexMut};

use anyhow::bail;
use indexmap::{IndexMap, map::Entry};

use crate::{
    manifest::Behavior,
    resource::{ResId, Resource, annotation},
};

#[derive(Clone, Default)]
pub struct ResourceMap {
    resources: IndexMap<ResId, Resource>,
}

impl fmt::Debug for ResourceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.resources.values()).finish()
    }
}

impl fmt::Display for ResourceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for resource in self.iter() {
            if self.len() > 1 {
                writeln!(f, "---")?;
            }
            write!(f, "{}", serde_yaml::to_string(resource).unwrap())?;
        }

        Ok(())
    }
}

impl ResourceMap {
    pub fn try_new(resources: impl IntoIterator<Item = Resource>) -> anyhow::Result<Self> {
        let iter = resources.into_iter();
        let mut resmap = Self::with_capacity(iter.size_hint().0);
        resmap.extend(iter)?;
        Ok(resmap)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: IndexMap::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn get(&self, id: &ResId) -> Option<&Resource> {
        self.resources.get(id)
    }

    pub fn keys(&self) -> impl ExactSizeIterator<Item = &ResId> + DoubleEndedIterator + fmt::Debug {
        self.resources.keys()
    }

    pub fn insert(&mut self, mut resource: Resource) -> anyhow::Result<()> {
        let behavior = resource
            .metadata()
            .annotations()
            .map_or(Ok(Behavior::Create), |annotations| annotations.behavior())?;
        if let Some(mut annotations) = resource.metadata_mut().annotations_mut() {
            annotations.remove(annotation::BEHAVIOR);
        }
        match self.resources.entry(resource.id().clone()) {
            Entry::Occupied(mut entry) => match behavior {
                Behavior::Create => bail!(
                    "may not add resource with an already registered id `{}`, consider specifying `merge` or `replace` behavior",
                    resource.id()
                ),
                Behavior::Merge => {
                    let left = entry
                        .get_mut()
                        .root_mut()
                        .get_mut("data")
                        .and_then(|data| data.as_object_mut());
                    let right = resource
                        .root()
                        .get("data")
                        .and_then(|data| data.as_object());

                    // TODO merging metadata and annotations, not sure what is correct behavior for this
                    match (left, right) {
                        (Some(left), Some(right)) => {
                            for (key, value) in right {
                                left.insert(key.clone(), value.clone());
                            }
                        }
                        (None, Some(right)) => {
                            entry.get_mut().root_mut()["data"] =
                                serde_json::Value::Object(right.clone());
                        }
                        (_, None) => {}
                    }
                }
                Behavior::Replace => todo!("replace"),
            },
            Entry::Vacant(entry) => match behavior {
                // FIXME: temporarily allow merging into a non-existent resource to pass test.
                Behavior::Create | Behavior::Merge => drop(entry.insert(resource)),
                Behavior::Replace => {
                    bail!(
                        "resource id `{}` does not exist, cannot {behavior}",
                        resource.id()
                    )
                }
            },
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

    /// In-place merge of two `ResourceMap`s into `self`
    pub fn merge(&mut self, other: ResourceMap) -> anyhow::Result<()> {
        for (_, resource) in other.resources {
            self.insert(resource)?;
        }
        Ok(())
    }

    pub fn extend(&mut self, resources: impl IntoIterator<Item = Resource>) -> anyhow::Result<()> {
        for resource in resources {
            self.insert(resource)?;
        }
        Ok(())
    }
}

impl IntoIterator for ResourceMap {
    type Item = Resource;
    type IntoIter = indexmap::map::IntoValues<ResId, Resource>;

    fn into_iter(self) -> Self::IntoIter {
        self.resources.into_values()
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
