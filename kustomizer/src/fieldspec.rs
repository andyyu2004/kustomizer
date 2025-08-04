mod builtin;

use core::fmt;
use std::str::FromStr;

pub use self::builtin::Builtin;

use crate::manifest::Str;
use serde::{Deserialize, Serialize};

// See kustomize/internal/konfig/builtinpluginconsts

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<Str>,
    #[serde(with = "crate::serde_ex::string")]
    pub path: Path,
    /// The `create` field indicates whether the field should be created if it does not exist.
    pub create: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    segments: Box<[PathSegment]>,
}

impl FromStr for Path {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = s
            .split('/')
            .map(|segment| segment.parse::<PathSegment>())
            .collect::<Result<_, _>>()?;
        Ok(Path { segments })
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .map(|segment| segment.to_string())
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    Field(Str),
    Array(Str),
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Field(field) => write!(f, "{field}"),
            PathSegment::Array(field) => write!(f, "{field}[]"),
        }
    }
}

impl FromStr for PathSegment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_suffix("[]") {
            Ok(PathSegment::Array(s.into()))
        } else {
            Ok(PathSegment::Field(s.into()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldSpecs {
    fields: Vec<FieldSpec>,
}

impl FieldSpecs {
    fn extend(&self, other: FieldSpecs) -> FieldSpecs {
        // TODO should check for overlaps
        let mut fields = self.fields.clone();
        fields.extend(other.fields);
        FieldSpecs { fields }
    }
}
