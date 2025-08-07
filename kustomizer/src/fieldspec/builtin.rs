use std::sync::OnceLock;

use serde::Deserialize;

use super::FieldSpecs;

const COMMON_ANNOTATIONS: &[u8] = include_bytes!("commonAnnotations.yaml");
const IMAGES: &[u8] = include_bytes!("images.yaml");
const METADATA_LABELS: &[u8] = include_bytes!("metadataLabels.yaml");
const TEMPLATE_LABELS: &[u8] = include_bytes!("templateLabels.yaml");
const OTHER_LABELS: &[u8] = include_bytes!("otherLabels.yaml");
const REPLICAS: &[u8] = include_bytes!("replicas.yaml");
const NAMESPACE: &[u8] = include_bytes!("namespace.yaml");
const NAME: &[u8] = include_bytes!("name.yaml");
const SUBJECTS: &[u8] = include_bytes!("subjects.yaml");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Builtin {
    pub name: FieldSpecs,
    pub namespace: FieldSpecs,
    pub metadata_labels: FieldSpecs,
    pub images: FieldSpecs,
    pub common_annotations: FieldSpecs,
    pub template_labels: FieldSpecs,
    pub common_labels: FieldSpecs,
    pub replicas: FieldSpecs,
    pub subjects: FieldSpecs,
}

impl Builtin {
    pub fn load() -> &'static Self {
        static INSTANCE: OnceLock<Builtin> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let template_labels =
                serde_yaml::from_slice::<FieldSpecs>(TEMPLATE_LABELS).expect("template labels");

            let other_labels =
                serde_yaml::from_slice::<FieldSpecs>(OTHER_LABELS).expect("other labels");

            let mut common_labels = template_labels.clone();
            common_labels
                .merge(other_labels)
                .expect("overlapping labels between template and other");

            Builtin {
                name: serde_yaml::from_slice::<FieldSpecs>(NAME).expect("name"),
                subjects: serde_yaml::from_slice::<FieldSpecs>(SUBJECTS)
                    .expect("subjects (other labels)"),
                namespace: serde_yaml::from_slice::<FieldSpecs>(NAMESPACE).expect("namespace"),
                common_annotations: serde_yaml::from_slice::<FieldSpecs>(COMMON_ANNOTATIONS)
                    .expect("common annotations"),
                common_labels,
                images: serde_yaml::from_slice::<FieldSpecs>(IMAGES).expect("images"),
                template_labels,
                replicas: serde_yaml::from_slice::<FieldSpecs>(REPLICAS).expect("replicas"),
                metadata_labels: serde_yaml::from_slice::<FieldSpecs>(METADATA_LABELS)
                    .expect("metadata labels"),
            }
        })
    }
}

#[cfg(test)]
#[test]
fn ensure_builtin_fieldspecs_valid() {
    Builtin::load();
}
