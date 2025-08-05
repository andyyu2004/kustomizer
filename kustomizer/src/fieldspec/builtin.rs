use std::sync::OnceLock;

use serde::Deserialize;

use super::FieldSpecs;

const COMMON_ANNOTATIONS: &[u8] = include_bytes!("commonAnnotations.yaml");
const METADATA_LABELS: &[u8] = include_bytes!("metadataLabels.yaml");
const TEMPLATE_LABELS: &[u8] = include_bytes!("templateLabels.yaml");
const OTHER_LABELS: &[u8] = include_bytes!("otherLabels.yaml");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Builtin {
    pub metadata_labels: FieldSpecs,
    pub common_annotations: FieldSpecs,
    pub template_labels: FieldSpecs,
    pub common_labels: FieldSpecs,
}

impl Builtin {
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<Builtin> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let common_annotations = serde_yaml::from_slice::<FieldSpecs>(COMMON_ANNOTATIONS)
                .expect("common annotations");

            let template_labels =
                serde_yaml::from_slice::<FieldSpecs>(TEMPLATE_LABELS).expect("template labels");

            let other_labels =
                serde_yaml::from_slice::<FieldSpecs>(OTHER_LABELS).expect("other labels");

            let common_labels = template_labels.extend(other_labels);

            Builtin {
                common_annotations,
                common_labels,
                metadata_labels: serde_yaml::from_slice::<FieldSpecs>(METADATA_LABELS)
                    .expect("metadata labels"),
                template_labels: serde_yaml::from_slice::<FieldSpecs>(TEMPLATE_LABELS)
                    .expect("template labels"),
            }
        })
    }
}

#[cfg(test)]
#[test]
fn ensure_builtin_fieldspecs_valid() {
    Builtin::get();
}
