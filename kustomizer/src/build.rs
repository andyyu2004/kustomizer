use std::collections::HashMap;

use crate::{Located, manifest::Kustomization};
use anyhow::Context;

// temporarily pub
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildContext {
    components: HashMap<std::path::PathBuf, crate::manifest::Component>,
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
        let path = base_path
            .join(path)
            .join("kustomization.yaml")
            .canonicalize()?;
        let component = crate::load_component(&path)
            .with_context(|| format!("loading component {}", path.display()))?;
        components.insert(component.path, component.value);
    }

    Ok(BuildContext { components })
}
