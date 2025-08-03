use crate::{manifest::Label, resmap::ResourceMap};

use super::Transformer;

pub struct LabelTransformer<'a>(pub &'a [Label]);

impl Transformer for LabelTransformer<'_> {
    fn transform(&mut self, resources: &mut ResourceMap) {
        if self.0.is_empty() {
            return;
        }

        for resource in resources.iter_mut() {
            for label in self.0 {
                // if label.include_selectors {
                //     todo!("include_selectors is not implemented");
                // }

                for (key, value) in &label.pairs {
                    resource.metadata.labels.insert(key.clone(), value.clone());
                }
            }
        }
    }
}
