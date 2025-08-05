use serde::Deserialize as _;

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

impl MetadataView<'_> {
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

impl AnnotationsView<'_> {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    pub fn behavior(&self) -> anyhow::Result<Behavior> {
        match self.get("kustomize.config.k8s.io/behavior") {
            Some(value) => serde_yaml::from_str(value)
                .map_err(|err| anyhow::anyhow!("failed to parse behavior: {err}")),
            None => Ok(Behavior::default()),
        }
    }

    pub fn function_spec(&self) -> anyhow::Result<Option<FunctionSpec>> {
        match self.get("config.kubernetes.io/function") {
            Some(yaml) => {
                let json = serde_yaml::from_str::<serde_json::Value>(yaml)?;
                FunctionSpec::deserialize(json)
                    .map_err(|err| anyhow::anyhow!("failed to deserialize function spec: {err}"))
                    .map(Some)
            }
            None => Ok(None),
        }
    }
}

pub struct MetadataViewMut<'a>(&'a mut serde_yaml::Mapping);

impl MetadataViewMut<'_> {
    pub fn annotations_mut(&mut self) -> Option<AnnotationsViewMut> {
        self.0
            .get_mut("annotations")
            .and_then(|v| v.as_mapping_mut())
            .map(AnnotationsViewMut)
    }

    pub fn labels_mut(&mut self) -> Option<LabelsViewMut<'_>> {
        self.0
            .get_mut("labels")
            .and_then(|v| v.as_mapping_mut())
            .map(LabelsViewMut)
    }

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<serde_yaml::Value> {
        self.0.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::String(value.into()),
        )
    }
}

pub struct LabelsViewMut<'a>(&'a mut serde_yaml::Mapping);

impl LabelsViewMut<'_> {
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<serde_yaml::Value> {
        self.0.insert(
            serde_yaml::Value::String(key.into()),
            serde_yaml::Value::String(value.into()),
        )
    }

    pub fn remove(&mut self, key: &str) {
        self.0.remove(serde_yaml::Value::String(key.to_string()));
    }
}

pub struct AnnotationsViewMut<'a>(&'a mut serde_yaml::Mapping);

impl AnnotationsViewMut<'_> {
    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(value.to_string()),
        );
    }

    pub fn remove(&mut self, key: &str) {
        self.0.remove(serde_yaml::Value::String(key.to_string()));
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
