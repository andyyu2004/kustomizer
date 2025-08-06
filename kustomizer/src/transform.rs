mod annotation;
mod function;
mod image;
mod label;
mod name;
mod namespace;
mod replica;

pub use self::annotation::AnnotationTransformer;
pub use self::image::ImageTagTransformer;
pub use self::label::LabelTransformer;
pub use self::name::NameTransformer;
pub use self::namespace::NamespaceTransformer;
pub use self::replica::ReplicaTransformer;

use crate::resmap::ResourceMap;

#[async_trait::async_trait]
pub trait Transformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()>;
}
