pub mod nested_yaml {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::manifest::Str;

    pub fn serialize<S, T>(patch: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let value = serde_json::to_value(patch).map_err(serde::ser::Error::custom)?;
        serde_yaml::to_string(&value)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let s: Str = Deserialize::deserialize(deserializer)?;
        // We convert the string to a JSON (not YAML) value to avoid enum weirdness with yaml tags.
        let yaml =
            serde_yaml::from_str::<serde_json::Value>(&s).map_err(serde::de::Error::custom)?;
        T::deserialize(yaml).map_err(serde::de::Error::custom)
    }
}
