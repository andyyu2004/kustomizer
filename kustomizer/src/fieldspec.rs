mod builtin;

use core::fmt;
use std::{ops::Deref, str::FromStr};

pub use self::builtin::Builtin;

use crate::{
    manifest::Str,
    resource::{AnyObject, GvkMatcher, Resource},
};
use anyhow::bail;
use serde::{Deserialize, Serialize};

// See kustomize/internal/konfig/builtinpluginconsts

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpec {
    #[serde(flatten)]
    pub matcher: GvkMatcher,
    #[serde(with = "crate::serde_ex::string")]
    pub path: FieldPath,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    /// The `create` field indicates whether the field should be created if it does not exist.
    pub create: bool,
}

impl FieldSpec {
    pub fn overlaps_with(&self, other: &FieldSpec) -> bool {
        self.matcher.overlaps_with(&other.matcher)
            && self.path == other.path
            && self.create == other.create
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPath {
    segments: Box<[FieldPathSegment]>,
}

impl AsRef<[FieldPathSegment]> for FieldPath {
    fn as_ref(&self) -> &[FieldPathSegment] {
        &self.segments
    }
}

impl Deref for FieldPath {
    type Target = [FieldPathSegment];

    fn deref(&self) -> &Self::Target {
        &self.segments
    }
}

pub type PathRef<'a> = &'a [FieldPathSegment];

impl<'a> IntoIterator for &'a FieldPath {
    type Item = &'a FieldPathSegment;
    type IntoIter = std::slice::Iter<'a, FieldPathSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.iter()
    }
}

impl FromStr for FieldPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = s
            .split('/')
            .map(|segment| segment.parse::<FieldPathSegment>())
            .collect::<Result<Box<_>, _>>()?;

        match segments.last() {
            Some(FieldPathSegment::Array(_)) => {
                Err(anyhow::anyhow!("path cannot end with an array segment"))?
            }
            None => Err(anyhow::anyhow!("path cannot be empty"))?,
            _ => Ok(FieldPath { segments }),
        }
    }
}

impl fmt::Display for FieldPath {
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
pub enum FieldPathSegment {
    Field(Str),
    Array(Str),
}

impl fmt::Display for FieldPathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldPathSegment::Field(field) => write!(f, "{field}"),
            FieldPathSegment::Array(field) => write!(f, "{field}[]"),
        }
    }
}

impl FromStr for FieldPathSegment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_suffix("[]") {
            Ok(FieldPathSegment::Array(s.into()))
        } else {
            Ok(FieldPathSegment::Field(s.into()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FieldSpecs {
    specs: Vec<FieldSpec>,
}

impl Deref for FieldSpecs {
    type Target = [FieldSpec];

    fn deref(&self) -> &Self::Target {
        &self.specs
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
    pub fn merge(&mut self, other: FieldSpecs) -> Result<(), Conflict> {
        for spec in other.specs {
            self.add(spec)?;
        }

        Ok(())
    }

    pub fn add(&mut self, spec: FieldSpec) -> Result<(), Conflict> {
        if let Some(conflicts_with) = self.specs.iter().find(|s| s.overlaps_with(&spec)) {
            Err(Conflict {
                conflicts_with: Box::new(conflicts_with.clone()),
                field_spec: Box::new(spec),
            })
        } else {
            self.specs.push(spec);
            Ok(())
        }
    }

    pub fn apply(
        &self,
        resource: &mut Resource,
        mut f: impl FnMut(&mut serde_json::Value) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for spec in &self.specs {
            spec.apply(resource, &mut f)?;
        }

        Ok(())
    }
}

impl FieldSpec {
    fn apply(
        &self,
        resource: &mut Resource,
        f: &mut impl FnMut(&mut serde_json::Value) -> anyhow::Result<()>,
    ) -> anyhow::Result<usize> {
        if !self.matcher.matches(resource.id()) {
            return Ok(0);
        }

        fn go(
            mut curr: &mut AnyObject,
            mut path: PathRef<'_>,
            f: &mut impl FnMut(&mut serde_json::Value) -> anyhow::Result<()>,
            create: bool,
        ) -> anyhow::Result<usize> {
            while let Some(segment) = path.first() {
                match segment {
                    FieldPathSegment::Field(field) => {
                        if !curr.contains_key(field.as_str()) {
                            if !create {
                                return Ok(0);
                            }

                            let new_value = serde_json::Value::Object(AnyObject::new());
                            curr.insert(field.to_string(), new_value);
                        }

                        let val = curr.get_mut(field.as_str()).unwrap();
                        if path.len() == 1 {
                            f(val)?;
                            return Ok(1);
                        }

                        curr = val.as_object_mut().ok_or_else(|| {
                            anyhow::anyhow!(
                                "expected a mapping at `{field}` but found a different type",
                            )
                        })?;
                    }
                    FieldPathSegment::Array(field) => {
                        match curr.get_mut(field.as_str()) {
                            Some(v) => match v.as_array_mut() {
                                Some(seq) => {
                                    let mut count = 0;
                                    for item in seq {
                                        if let Some(map) = item.as_object_mut() {
                                            count += go(map, &path[1..], f, create)?;
                                        }
                                    }

                                    return Ok(count);
                                }
                                None => bail!(
                                    "expected a sequence at `{field}` but found a different type",
                                ),
                            },
                            // No point creating an empty array, so `create` has no effect here.
                            None => return Ok(0),
                        }
                    }
                }
                path = &path[1..];
            }

            Ok(0)
        }

        go(resource.root_mut(), &self.path, f, self.create)
    }
}
