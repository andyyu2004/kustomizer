use std::sync::OnceLock;

use serde::Deserialize;

use super::FieldSpecs;

const COMMON_ANNOTATIONS: &[u8] = include_bytes!("commonAnnotations.yaml");
const METADATA_LABELS: &[u8] = include_bytes!("metadataLabels.yaml");
const OTHER_LABELS: &[u8] = include_bytes!("otherLabels.yaml");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Builtin {
    pub common_annotations: FieldSpecs,
    pub metadata_labels: FieldSpecs,
    pub common_labels: FieldSpecs,
}

impl Builtin {
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<Builtin> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let common_annotations = serde_yaml::from_slice::<FieldSpecs>(COMMON_ANNOTATIONS)
                .expect("common annotations");
            let metadata_labels =
                serde_yaml::from_slice::<FieldSpecs>(METADATA_LABELS).expect("metadata labels");

            let other_labels =
                serde_yaml::from_slice::<FieldSpecs>(OTHER_LABELS).expect("other labels");

            let common_labels = metadata_labels.extend(other_labels);

            Builtin {
                common_annotations,
                metadata_labels,
                common_labels,
            }
        })
    }
}

#[cfg(test)]
#[test]
fn ensure_builtin_fieldspecs_valid() {
    eprintln!("{:#?}", Builtin::get())
}
