use serde::Deserialize as _;
use serde_json::map::Entry;

use crate::{
    manifest::{Behavior, FunctionSpec},
    yaml,
};

use super::{Object, Resource, annotation};

impl Resource {
    pub fn metadata(&self) -> Option<MetadataView<'_>> {
        self.root
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(MetadataView)
    }

    pub fn labels(&self) -> Option<LabelsView<'_>> {
        self.metadata()?.labels()
    }

    pub fn annotations(&self) -> Option<AnnotationsView<'_>> {
        self.metadata()?.annotations()
    }

    pub fn make_metadata_mut(&mut self) -> MetadataViewMut<'_> {
        let root = self.root.as_object_mut().unwrap();
        if !root.contains_key("metadata") {
            root.insert(
                "metadata".to_string(),
                serde_json::Value::Object(Object::new()),
            );
        }

        self.metadata_mut().unwrap()
    }

    pub fn metadata_mut(&mut self) -> Option<MetadataViewMut<'_>> {
        self.root
            .get_mut("metadata")
            .and_then(|v| v.as_object_mut())
            .map(MetadataViewMut)
    }
}

#[derive(Debug)]
pub struct MetadataView<'a>(&'a Object);

impl<'a> MetadataView<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.0.get("name").and_then(|v| v.as_str())
    }

    pub fn namespace(&self) -> Option<&'a str> {
        self.0.get("namespace").and_then(|v| v.as_str())
    }

    pub fn annotations(&self) -> Option<AnnotationsView<'a>> {
        self.0
            .get("annotations")
            .and_then(|v| v.as_object())
            .map(AnnotationsView)
    }

    pub fn labels(&self) -> Option<LabelsView<'a>> {
        self.0
            .get("labels")
            .and_then(|v| v.as_object())
            .map(LabelsView)
    }
}

#[derive(Debug)]
pub struct LabelsView<'a>(&'a Object);

impl<'a> LabelsView<'a> {
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
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

#[derive(Debug)]
pub struct AnnotationsView<'a>(&'a Object);

impl<'a> AnnotationsView<'a> {
    pub fn get(&self, key: &str) -> Option<&'a str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
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

    pub fn behavior(&self) -> anyhow::Result<Behavior> {
        match self.get("kustomize.config.k8s.io/behavior") {
            Some(value) => yaml::from_str(value)
                .map_err(|err| anyhow::anyhow!("failed to parse behavior: {err}")),
            None => Ok(Behavior::default()),
        }
    }

    pub fn needs_hash(&self) -> bool {
        matches!(self.get(annotation::NEEDS_HASH), Some(v) if v == "true")
    }

    pub fn function_spec(&self) -> anyhow::Result<Option<FunctionSpec>> {
        match self.get(annotation::FUNCTION) {
            Some(yaml) => {
                let json = yaml::from_str::<serde_json::Value>(yaml)?;
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
    // This is private because it is unsafe to be used alone since the id must also be modified alongside.
    pub(super) fn set_name(&mut self, name: impl Into<String>) {
        self.0
            .insert("name".to_string(), serde_json::Value::String(name.into()));
    }

    // This is private because it is unsafe to be used alone since the id must also be modified alongside.
    pub(super) fn set_namespace(&mut self, namespace: Option<impl Into<String>>) {
        match namespace {
            None => self.0.remove("namespace"),
            Some(namespace) => self.0.insert(
                "namespace".to_string(),
                serde_json::Value::String(namespace.into()),
            ),
        };
    }

    pub fn make_annotations_mut(&mut self) -> AnnotationsViewMut<'_> {
        if !self.0.contains_key("annotations") {
            self.0.insert(
                "annotations".to_string(),
                serde_json::Value::Object(Object::new()),
            );
        }
        self.annotations_mut().unwrap()
    }

    pub(crate) fn clear_internal_fields(&mut self) {
        if let Some(mut annotations) = self.annotations_mut() {
            annotations.remove(annotation::BEHAVIOR);
            annotations.remove(annotation::NEEDS_HASH);
            annotations.remove(annotation::FUNCTION);
            annotations.remove(annotation::PREVIOUS_KINDS);
            annotations.remove(annotation::PREVIOUS_NAMESPACES);
            annotations.remove(annotation::PREVIOUS_NAMES);
        }
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

    pub fn make_labels_mut(&mut self) -> LabelsViewMut<'_> {
        if !self.0.contains_key("labels") {
            self.0.insert(
                "labels".to_string(),
                serde_json::Value::Object(Object::new()),
            );
        }
        self.labels_mut().unwrap()
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
    pub fn insert(&mut self, key: impl Into<String>, value: &str) {
        self.0
            .insert(key.into(), serde_json::Value::String(value.to_string()));
    }

    pub fn insert_or_update(&mut self, key: impl Into<String>, f: impl FnOnce(&mut String)) {
        let key = key.into();
        let v = match self.0.entry(key) {
            Entry::Vacant(entry) => {
                let v = entry.insert(serde_json::Value::String(String::new()));
                let serde_json::Value::String(v) = v else {
                    unreachable!();
                };
                v
            }
            Entry::Occupied(entry) => {
                let v = entry.into_mut();
                let serde_json::Value::String(v) = v else {
                    panic!("got non-string annotation value")
                };
                v
            }
        };
        f(v);
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        match self.0.remove(key)? {
            serde_json::Value::String(v) => Some(v),
            _ => None,
        }
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

    pub fn set_needs_hash(&mut self) {
        self.insert(annotation::NEEDS_HASH, "true");
    }
}
