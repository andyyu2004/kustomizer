mod annotation;
mod cleanup;
mod function;
mod image;
mod label;
mod name;
mod namespace;
mod patch;
mod replica;

pub use self::annotation::AnnotationTransformer;
pub use self::cleanup::CleanupTransformer;
pub use self::image::ImageTagTransformer;
pub use self::label::LabelTransformer;
pub use self::name::NameTransformer;
pub use self::namespace::NamespaceTransformer;
pub use self::patch::PatchTransformer;
pub use self::replica::ReplicaTransformer;

use crate::resmap::ResourceMap;

pub trait Transformer {
    async fn transform(&mut self, resources: &mut ResourceMap) -> anyhow::Result<()>;
}
