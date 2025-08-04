use crate::manifest::{Behavior, FunctionSpec};

use super::Resource;

impl Resource {
    pub fn metadata(&self) -> MetadataView<'_> {
        MetadataView(self.root["metadata"].as_mapping().unwrap())
    }

    pub fn metadata_mut(&mut self) -> MetadataViewMut<'_> {
        MetadataViewMut(self.root["metadata"].as_mapping_mut().unwrap())
    }
}

pub struct MetadataView<'a>(&'a serde_yaml::Mapping);

impl<'a> MetadataView<'a> {
    pub fn annotations(&self) -> Option<AnnotationsView<'_>> {
        self.0
            .get("annotations")
            .and_then(|v| v.as_mapping())
            .map(AnnotationsView)
    }

    pub fn labels(&self) -> Option<&serde_yaml::Mapping> {
        self.0.get("labels").and_then(|v| v.as_mapping())
    }
}

pub struct AnnotationsView<'a>(&'a serde_yaml::Mapping);

impl<'a> AnnotationsView<'a> {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .get(&serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
    }

    pub fn behavior(&self) -> anyhow::Result<Behavior> {
        match self.get("config.kubernetes.io/behavior") {
            Some(value) => serde_yaml::from_str(value)
                .map_err(|err| anyhow::anyhow!("failed to parse behavior: {err}")),
            None => Ok(Behavior::default()),
        }
    }

    pub fn function_spec(&self) -> anyhow::Result<Option<FunctionSpec>> {
        match self.get("config.kubernetes.io/function") {
            Some(yaml) => serde_yaml::from_str(yaml)
                .map_err(|err| anyhow::anyhow!("failed to parse function spec: {err}"))
                .map(Some),
            None => Ok(None),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.0.iter().filter_map(|(k, v)| {
            if let (serde_yaml::Value::String(key), serde_yaml::Value::String(value)) = (k, v) {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
    }
}

pub struct MetadataViewMut<'a>(&'a mut serde_yaml::Mapping);

impl<'a> MetadataViewMut<'a> {
    pub fn annotations_mut(&mut self) -> Option<AnnotationsViewMut> {
        self.0
            .get_mut("annotations")
            .and_then(|v| v.as_mapping_mut())
            .map(AnnotationsViewMut::new)
    }

    pub fn labels_mut(&mut self) -> Option<&mut serde_yaml::Mapping> {
        self.0.get_mut("labels").and_then(|v| v.as_mapping_mut())
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::String(value.into()),
        );
    }
}

pub struct AnnotationsViewMut<'a> {
    annotations: &'a mut serde_yaml::Mapping,
}

impl<'a> AnnotationsViewMut<'a> {
    fn new(annotations: &'a mut serde_yaml::Mapping) -> Self {
        AnnotationsViewMut { annotations }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.annotations.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(value.to_string()),
        );
    }

    pub fn remove(&mut self, key: &str) {
        self.annotations
            .remove(&serde_yaml::Value::String(key.to_string()));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.annotations.iter().filter_map(|(k, v)| {
            if let (serde_yaml::Value::String(key), serde_yaml::Value::String(value)) = (k, v) {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
    }
}
