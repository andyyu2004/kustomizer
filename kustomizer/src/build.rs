use std::collections::HashMap;

use crate::{
    Located, PathId,
    manifest::{Component, Kustomization},
};
use anyhow::Context;

// temporarily pub
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildContext {
    components: HashMap<PathId, Component>,
}

// temporarily pub
pub async fn gather_context(
    kustomization: &Located<Kustomization>,
) -> anyhow::Result<BuildContext> {
    let base_path = kustomization
        .path
        .parent()
        .expect("this is a file so it has a parent")
        .canonicalize()?;
    let mut components = HashMap::with_capacity(kustomization.components.len());
    for path in &kustomization.components {
        let component = crate::load_component(base_path.join(path))
            .with_context(|| format!("loading component {}", path.display()))?;
        components.insert(component.path, component.value);
    }

    Ok(BuildContext { components })
}
