mod builtin;

use core::fmt;
use std::{ops::Deref, str::FromStr};

pub use self::builtin::Builtin;

use crate::{
    manifest::Str,
    resource::{GvkMatcher, Object, Resource},
};
use anyhow::{Context as _, bail};
use json::Value;
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

#[derive(Clone, PartialEq, Eq)]
pub struct FieldPath {
    segments: Box<[FieldPathSegment]>,
}

impl fmt::Debug for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{self}""#)
    }
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

    // TODO Need to handle escaping of '/' by '\'
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = s
            .split('/')
            .map(|segment| segment.parse::<FieldPathSegment>())
            .collect::<Result<Box<_>, _>>()?;

        match segments.last() {
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

    pub fn apply<T: JsonValue>(
        &self,
        resource: &mut Resource,
        mut f: impl FnMut(&mut T) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for spec in &self.specs {
            spec.apply(resource, &mut f)?;
        }

        Ok(())
    }
}

impl FieldSpec {
    pub fn apply<T>(
        &self,
        resource: &mut Resource,
        f: &mut impl FnMut(&mut T) -> anyhow::Result<()>,
    ) -> anyhow::Result<()>
    where
        T: JsonValue,
    {
        if !self.matcher.matches(resource.id()) {
            return Ok(());
        }

        fn go<T>(
            mut curr: &mut Value,
            mut path: PathRef<'_>,
            f: &mut impl FnMut(&mut T) -> anyhow::Result<()>,
            create: bool,
        ) -> anyhow::Result<()>
        where
            T: JsonValue,
        {
            while let Some(segment) = path.first() {
                match segment {
                    FieldPathSegment::Field(field) => {
                        let obj = curr.as_object_mut().ok_or_else(|| {
                            let path = path
                                .iter()
                                .map(|s| s.to_string())
                                .collect::<Vec<_>>()
                                .join("/");
                            anyhow::anyhow!("expected an object value at `{path}/{field}`")
                        })?;

                        if !obj.contains_key(field.as_str()) {
                            if !create {
                                return Ok(());
                            }

                            obj.insert(field.to_string(), T::default().into_value());
                        }

                        let val = obj.get_mut(field.as_str()).unwrap();
                        if path.len() == 1 {
                            f(T::try_as_mut(val)?)?;
                            return Ok(());
                        }

                        curr = val;
                    }
                    FieldPathSegment::Array(field) => match curr.get_mut(field.as_str()) {
                        Some(v) => match v {
                            Value::Null => return Ok(()),
                            Value::Array(seq) => {
                                for item in seq {
                                    go(item, &path[1..], f, create)?;
                                }

                                return Ok(());
                            }
                            _ => {
                                return Err(anyhow::anyhow!(
                                    "expected a sequence at `{}` but found a value of different type",
                                    field
                                ));
                            }
                        },
                        None => return Ok(()),
                    },
                }
                path = &path[1..];
            }

            f(T::try_as_mut(curr)?)?;

            Ok(())
        }

        go(resource.root_raw_mut(), &self.path, f, self.create).with_context(|| {
            format!(
                "applying field spec `{}` `{}` to resource {}",
                self.matcher,
                self.path,
                resource.id()
            )
        })
    }
}

pub trait JsonValue: Default {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self>;

    fn into_value(self) -> Value
    where
        Self: Sized;
}

impl JsonValue for Value {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self> {
        Ok(value)
    }

    fn into_value(self) -> Value {
        self
    }
}

impl JsonValue for Object {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self> {
        match value {
            Value::Object(obj) => Ok(obj),
            _ => bail!("expected an object but found a different type"),
        }
    }

    fn into_value(self) -> Value {
        Value::Object(self)
    }
}

impl JsonValue for Vec<Value> {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self> {
        match value {
            Value::Array(arr) => Ok(arr),
            _ => bail!("expected a sequence but found a different type"),
        }
    }

    fn into_value(self) -> Value {
        Value::Array(self)
    }
}

impl JsonValue for String {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => bail!("expected a string but found a different type"),
        }
    }

    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl JsonValue for bool {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => bail!("expected a boolean but found a different type"),
        }
    }

    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl JsonValue for u64 {
    fn try_as_mut(value: &mut Value) -> anyhow::Result<&mut Self> {
        match value {
            Value::Number(num) if num.is_u64() => Ok(num.as_u64_mut().unwrap()),
            _ => bail!("expected a unsigned number but found a different type"),
        }
    }

    fn into_value(self) -> Value {
        Value::Number(json::Number::from(self))
    }
}
