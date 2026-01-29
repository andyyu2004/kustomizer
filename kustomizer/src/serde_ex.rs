pub mod regex {
    use ::regex::Regex;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(re: &Regex, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(re)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        super::anchored_regex(&s).map_err(serde::de::Error::custom)
    }
}

fn anchored_regex(s: &str) -> Result<::regex::Regex, ::regex::Error> {
    if s.starts_with('^') && s.ends_with('$') {
        ::regex::Regex::new(s)
    } else if s.starts_with('^') {
        ::regex::Regex::new(&format!("{s}$"))
    } else if s.ends_with('$') {
        ::regex::Regex::new(&format!("^{s}"))
    } else {
        ::regex::Regex::new(&format!("^{s}$"))
    }
}

pub mod opt_regex {
    use regex::Regex;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(re: &Option<Regex>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match re {
            Some(re) => super::regex::serialize(re, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Deserialize::deserialize(deserializer)?;
        match opt {
            Some(s) => super::anchored_regex(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

pub mod nested_yaml {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::{manifest::Str, yaml};

    pub fn serialize<S, T>(patch: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let value = serde_json::to_value(patch).map_err(serde::ser::Error::custom)?;
        yaml::to_string(&value)
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
        let yaml = yaml::from_str::<serde_json::Value>(&s).map_err(serde::de::Error::custom)?;
        T::deserialize(yaml).map_err(serde::de::Error::custom)
    }
}

pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
