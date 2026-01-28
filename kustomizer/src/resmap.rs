use core::fmt;
use std::ops::{Index, IndexMut};

use anyhow::{Context, bail};
use indexmap::IndexMap;

use crate::{
    manifest::Behavior,
    resource::{ResId, Resource},
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

    pub fn insert(&mut self, resource: Resource) -> anyhow::Result<()> {
        let mut matches = self
            .resources
            .values_mut()
            .filter(|res| res.any_id_matches(|id| *resource.id() == id))
            .fuse();

        let fst_match = matches.next();
        let snd_match = matches.next();

        let behavior = resource
            .annotations()
            .map_or(Ok(Behavior::Create), |annotations| annotations.behavior())?;

        match (fst_match, snd_match) {
            (None, _) => match behavior {
                // FIXME bug
                // Merge should cause failure if the resource does not already exist, but causes a
                // test failure currently.
                Behavior::Create | Behavior::Merge => {
                    drop(self.resources.insert(resource.id().clone(), resource))
                }
                Behavior::Replace => panic!(
                    "resource id `{}` does not exist, cannot {behavior}",
                    resource.id()
                ),
            },
            (Some(existing), None) => match behavior {
                Behavior::Create => bail!(
                    "may not add resource with an already registered id `{}`, consider specifying `merge` or `replace` behavior",
                    resource.id()
                ),
                Behavior::Merge => existing.merge_data_fields(resource).with_context(|| {
                    format!("failed to merge resources with id `{}`", existing.id())
                })?,
                Behavior::Replace => {
                    let existing_id = existing.id().clone();
                    let resource = if let Some(ns) = existing_id.namespace.clone() {
                        resource.with_namespace(ns)
                    } else {
                        resource
                    };

                    assert!(self.resources.insert(existing_id, resource).is_some());
                }
            },
            _ => {
                bail!(
                    "multiple resources match the id `{}` that could accept merge",
                    resource.id()
                )
            }
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
