use serde::Deserialize as _;

use crate::manifest::{Behavior, FunctionSpec};

use super::{Object, Resource};

impl Resource {
    pub fn metadata(&self) -> MetadataView<'_> {
        MetadataView(self.root["metadata"].as_object().unwrap())
    }

    pub fn labels(&self) -> Option<&Object> {
        self.metadata().labels()
    }

    pub fn annotations(&self) -> Option<AnnotationsView<'_>> {
        self.metadata().annotations()
    }

    pub fn metadata_mut(&mut self) -> MetadataViewMut<'_> {
        MetadataViewMut(self.root["metadata"].as_object_mut().unwrap())
    }
}

#[derive(Debug)]
pub struct MetadataView<'a>(&'a Object);

impl<'a> MetadataView<'a> {
    pub fn annotations(&self) -> Option<AnnotationsView<'a>> {
        self.0
            .get("annotations")
            .and_then(|v| v.as_object())
            .map(AnnotationsView)
    }

    pub fn labels(&self) -> Option<&'a Object> {
        self.0.get("labels").and_then(|v| v.as_object())
    }
}

#[derive(Debug)]
pub struct AnnotationsView<'a>(&'a Object);

impl AnnotationsView<'_> {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
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

#[derive(Debug)]
pub struct MetadataViewMut<'a>(&'a mut Object);

impl MetadataViewMut<'_> {
    pub(crate) fn clear_internal_fields(&mut self) {
        if let Some(mut annotations) = self.annotations_mut() {
            annotations.remove("config.kubernetes.io/function");
            annotations.remove("kustomize.config.k8s.io/behavior");
        }
    }

    // This is private because it is unsafe to be used alone since the id must also be modified alongside.
    pub(super) fn set_name(&mut self, name: impl Into<String>) {
        self.0
            .insert("name".to_string(), serde_json::Value::String(name.into()));
    }

    // This is private because it is unsafe to be used alone since the id must also be modified alongside.
    pub(super) fn set_namespace(&mut self, namespace: impl Into<String>) {
        self.0.insert(
            "namespace".to_string(),
            serde_json::Value::String(namespace.into()),
        );
    }

    pub fn annotations_mut(&mut self) -> Option<AnnotationsViewMut<'_>> {
        self.0
            .get_mut("annotations")
            .and_then(|v| v.as_object_mut())
            .map(AnnotationsViewMut)
    }

    pub fn labels_mut(&mut self) -> Option<LabelsViewMut<'_>> {
        self.0
            .get_mut("labels")
            .and_then(|v| v.as_object_mut())
            .map(LabelsViewMut)
    }

    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<serde_json::Value> {
        self.0
            .insert(key.into(), serde_json::Value::String(value.into()))
    }
}

#[derive(Debug)]
pub struct LabelsViewMut<'a>(&'a mut Object);

impl LabelsViewMut<'_> {
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Option<serde_json::Value> {
        self.0
            .insert(key.into(), serde_json::Value::String(value.into()))
    }

    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }
}

#[derive(Debug)]
pub struct AnnotationsViewMut<'a>(&'a mut Object);

impl AnnotationsViewMut<'_> {
    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }

    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.0.iter().filter_map(|(k, v)| {
            if let (key, serde_json::Value::String(value)) = (k, v) {
                Some((key.as_str(), value.as_str()))
            } else {
                None
            }
        })
    }
}
