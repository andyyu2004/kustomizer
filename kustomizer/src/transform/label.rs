use core::fmt;
use std::borrow::Cow;

use indexmap::IndexMap;

use crate::{
    fieldspec,
    manifest::{Label, Str},
    resmap::ResourceMap,
    resource::Object,
};

use super::Transformer;

#[derive(Debug)]
pub struct LabelTransformer<'a> {
    labels: Cow<'a, [Label]>,
}

impl fmt::Display for LabelTransformer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.labels.iter().flat_map(|label| label.pairs.iter()))
            .finish()
    }
}

impl<'a, 'de> serde::Deserialize<'de> for LabelTransformer<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Labels {
            labels: IndexMap<Str, Str>,
        }

        let labels = Labels::deserialize(deserializer)?;
        Ok(LabelTransformer::new(vec![Label {
            pairs: labels.labels,
            include_selectors: Default::default(),
            include_templates: Default::default(),
        }]))
    }
}

impl<'a> LabelTransformer<'a> {
    pub fn new(labels: impl Into<Cow<'a, [Label]>>) -> Self {
        Self {
            labels: labels.into(),
        }
    }
}

impl Transformer for LabelTransformer<'_> {
    #[tracing::instrument(skip_all, name = "label_transform", fields(labels = %self))]
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()> {
        if self.labels.is_empty() {
            return Ok(());
        }

        let builtins = fieldspec::Builtin::load();

        for label in &self.labels[..] {
            if label.pairs.is_empty() {
                continue;
            }

            let field_specs = match (label.include_selectors, label.include_templates) {
                (true, _) => &builtins.common_labels,
                (false, true) => &builtins.template_labels,
                (false, false) => &builtins.metadata_labels,
            };

            for resource in resources.iter_mut() {
                field_specs.apply::<Object>(resource, |l| {
                    for (key, value) in &label.pairs {
                        l.insert(key.to_string(), json::Value::String(value.to_string()));
                    }

                    Ok(())
                })?;
            }
        }

        Ok(())
    }
}
