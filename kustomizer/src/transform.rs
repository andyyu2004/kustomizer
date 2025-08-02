mod annotation;

pub use self::annotation::AnnotationTransformer;

use crate::resmap::ResourceMap;

pub trait Transformer {
    fn transform(&mut self, resources: &mut ResourceMap);
}
