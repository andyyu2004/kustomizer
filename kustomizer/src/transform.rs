mod annotation;
mod label;
mod namespace;

pub use self::annotation::AnnotationTransformer;
pub use self::label::LabelTransformer;
pub use self::namespace::NamespaceTransformer;

use crate::resmap::ResourceMap;

pub trait Transformer {
    fn transform(&mut self, resources: &mut ResourceMap);
}
