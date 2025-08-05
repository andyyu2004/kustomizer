mod annotation;
mod function;
mod label;
mod namespace;

pub use self::annotation::AnnotationTransformer;
pub use self::label::LabelTransformer;
pub use self::namespace::NamespaceTransformer;

use crate::resmap::ResourceMap;

#[async_trait::async_trait]
pub trait Transformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()>;
}
