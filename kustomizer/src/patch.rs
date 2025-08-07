use crate::resource::Resource;

mod openapi;

pub fn apply(base: &mut Resource, patch: Resource) -> anyhow::Result<()> {
    let spec = openapi::v2::Spec::load();
    Ok(())
}
