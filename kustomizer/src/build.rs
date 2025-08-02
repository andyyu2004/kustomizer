use crate::manifest::Kustomization;

struct BuildContext {}

async fn gather_context(kustomization: &Kustomization) -> anyhow::Result<BuildContext> {
    Ok(BuildContext {})
}
