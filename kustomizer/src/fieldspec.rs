mod builtin;

use core::fmt;
use std::{ops::Deref, str::FromStr};

pub use self::builtin::Builtin;

use crate::{
    manifest::Str,
    resource::{GvkMatcher, Resource},
};
use serde::{Deserialize, Serialize};

// See kustomize/internal/konfig/builtinpluginconsts

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpec {
    #[serde(flatten)]
    pub matcher: GvkMatcher,
    #[serde(with = "crate::serde_ex::string")]
    pub path: Path,
    /// The `create` field indicates whether the field should be created if it does not exist.
    pub create: bool,
}

impl FieldSpec {
    fn overlaps_with(&self, other: &FieldSpec) -> bool {
        self.matcher.overlaps_with(&other.matcher)
            && self.path == other.path
            && self.create == other.create
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    segments: Box<[PathSegment]>,
}

impl AsRef<[PathSegment]> for Path {
    fn as_ref(&self) -> &[PathSegment] {
        &self.segments
    }
}

impl Deref for Path {
    type Target = [PathSegment];

    fn deref(&self) -> &Self::Target {
        &self.segments
    }
}

pub type PathRef<'a> = &'a [PathSegment];

impl<'a> IntoIterator for &'a Path {
    type Item = &'a PathSegment;
    type IntoIter = std::slice::Iter<'a, PathSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.iter()
    }
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
pub enum PathSegment {
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

impl IntoIterator for FieldSpecs {
    type Item = FieldSpec;
    type IntoIter = std::vec::IntoIter<FieldSpec>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl Deref for FieldSpecs {
    type Target = [FieldSpec];

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

#[derive(Debug)]
pub struct Conflict {
    pub conflicts_with: Box<FieldSpec>,
    pub field_spec: Box<FieldSpec>,
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot add field spec `{}` because it conflicts with existing field spec `{}`",
            self.field_spec.matcher, self.conflicts_with.matcher
        )
    }
}

impl std::error::Error for Conflict {}

impl FieldSpecs {
    pub fn merge(&mut self, specs: impl IntoIterator<Item = FieldSpec>) -> Result<(), Conflict> {
        for spec in specs {
            self.add(spec)?;
        }

        Ok(())
    }

    pub fn add(&mut self, spec: FieldSpec) -> Result<(), Conflict> {
        if let Some(conflicts_with) = self.fields.iter().find(|s| s.overlaps_with(&spec)) {
            Err(Conflict {
                conflicts_with: Box::new(conflicts_with.clone()),
                field_spec: Box::new(spec),
            })
        } else {
            self.fields.push(spec);
            Ok(())
        }
    }
}

impl FieldSpec {
    pub fn apply(&self, resource: &mut Resource, f: impl FnMut(&mut serde_yaml::Mapping) + Copy) {
        if !self.matcher.matches(resource.id()) {
            return;
        }

        fn go(
            mut curr: &mut serde_yaml::Mapping,
            mut path: PathRef<'_>,
            mut f: impl FnMut(&mut serde_yaml::Mapping) + Copy,
            create: bool,
        ) -> Option<()> {
            while let Some(segment) = path.first() {
                match segment {
                    PathSegment::Field(field) => {
                        if !curr.contains_key(field.as_str()) {
                            if !create {
                                return None;
                            }

                            let new_value = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
                            curr.insert(serde_yaml::Value::String(field.to_string()), new_value);
                        }

                        curr = curr.get_mut(field.as_str())?.as_mapping_mut()?;
                    }
                    PathSegment::Array(field) => {
                        // TODO handle `create` in this case maybe?
                        let seq = curr
                            .get_mut(field.as_str())
                            .and_then(|v| v.as_sequence_mut())?;
                        for item in seq {
                            if let Some(map) = item.as_mapping_mut() {
                                go(map, &path[1..], f, create);
                            }
                        }
                    }
                }
                path = &path[1..];
            }

            f(curr);
            Some(())
        }

        go(resource.root_mut(), &self.path, f, self.create);
    }
}
