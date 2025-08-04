use indexmap::IndexMap;

use crate::{fieldspec, manifest::Str};

use super::{ResourceMap, Transformer};

// This implementation is not right, see following. It should take some well known paths and only patch those.
// internal/konfig/builtinpluginconsts/commonannotations.go
pub struct AnnotationTransformer<'a>(pub &'a IndexMap<Str, Str>);

impl Transformer for AnnotationTransformer<'_> {
    fn transform(&mut self, resources: &mut ResourceMap) {
        if self.0.is_empty() {
            return;
        }

        let _fieldspecs = &fieldspec::Builtin::get().common_annotations;
        dbg!(_fieldspecs);

        // for resource in resources.iter_mut() {
        //     resource
        //         .metadata_mut()
        //         .annotations_mut()
        //         .extend(self.0.iter().map(|(k, v)| (k.clone(), v.clone())));
        //
        //     resource.root.visit_with(self);
        // }
    }
}
