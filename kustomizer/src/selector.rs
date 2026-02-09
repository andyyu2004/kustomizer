use core::fmt;
use std::str::FromStr;

use anyhow::bail;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use crate::manifest::Str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Equality(Str, Str),
    Inequality(Str, Str),
    SetInclusion(Str, IndexSet<Str>),
    SetExclusion(Str, IndexSet<Str>),
    Existence(Str),
    All(Vec<Selector>),
}

pub trait StringMap {
    fn get(&self, key: &str) -> Option<&str>;

    fn has(&self, key: &str) -> bool;
}

impl<T: StringMap + ?Sized> StringMap for &T {
    fn get(&self, key: &str) -> Option<&str> {
        (*self).get(key)
    }

    fn has(&self, key: &str) -> bool {
        (*self).has(key)
    }
}

impl Selector {
    pub(crate) fn matches(&self, map: Option<&impl StringMap>) -> bool {
        let Some(m) = map else { return false };
        match self {
            Selector::Equality(key, value) => m.get(key).is_some_and(|v| v == value),
            Selector::Inequality(key, value) => m.get(key).is_some_and(|v| v != value),
            Selector::SetInclusion(key, values) => m.get(key).is_some_and(|v| values.contains(v)),
            Selector::SetExclusion(key, values) => m.get(key).is_some_and(|v| !values.contains(v)),
            Selector::Existence(key) => m.has(key),
            Selector::All(selectors) => selectors.iter().all(|s| s.matches(map)),
        }
    }
}

struct Parser<'s> {
    lexer: Lexer<'s>,
}

impl Parser<'_> {
    fn op_one(
        &mut self,
        operator: &str,
        base: Str,
        f: impl FnOnce(Str, Str) -> Selector,
    ) -> anyhow::Result<Selector> {
        if let Some(Token::Ident(value)) = self.lexer.next().transpose()? {
            Ok(f(base, value))
        } else {
            bail!("expected identifier after operator `{operator}`");
        }
    }

    fn op_many(
        &mut self,
        operator: &str,
        base: Str,
        f: impl FnOnce(Str, IndexSet<Str>) -> Selector,
    ) -> anyhow::Result<Selector> {
        let Some(Token::LeftParen) = self.lexer.next().transpose()? else {
            bail!("expected `(` after operator `{operator}`");
        };

        let mut values = IndexSet::new();
        loop {
            match self.lexer.next().transpose()? {
                Some(Token::Ident(value)) => {
                    values.insert(value);
                    match self.lexer.next().transpose()? {
                        Some(Token::Comma) => continue,
                        Some(Token::RightParen) => break,
                        _ => bail!("expected `,` or `)` after value"),
                    }
                }
                _ => bail!("expected identifier in set"),
            }
        }

        Ok(f(base, values))
    }

    fn parse(&mut self) -> anyhow::Result<Selector> {
        let mut selectors = Vec::new();
        loop {
            let key = match self.lexer.next().transpose()? {
                Some(Token::Ident(key)) => key,
                Some(token) => bail!("unexpected token `{token:?}` at start of selector"),
                None => break,
            };

            let selector = match self.lexer.next() {
                Some(token) => match token? {
                    Token::Ident(_) => bail!("unexpected identifier after selector key"),
                    Token::Equal => self.op_one("=", key, Selector::Equality)?,
                    Token::NotEqual => self.op_one("!=", key, Selector::Inequality)?,
                    Token::In => self.op_many("in", key, Selector::SetInclusion)?,
                    Token::NotIn => self.op_many("notin", key, Selector::SetExclusion)?,
                    Token::Comma => bail!("unexpected `,` after selector key"),
                    Token::LeftParen => bail!("unexpected `(` after selector key"),
                    Token::RightParen => bail!("unexpected `)` after selector key"),
                },
                None => Selector::Existence(key),
            };
            selectors.push(selector);

            match self.lexer.next().transpose()? {
                Some(Token::Comma) => continue,
                Some(token) => bail!("unexpected token `{token:?}` after selector"),
                None => break,
            }
        }

        if selectors.len() == 1 {
            Ok(selectors.into_iter().next().unwrap())
        } else {
            Ok(Selector::All(selectors))
        }
    }
}

impl FromStr for Selector {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser {
            lexer: Lexer { input },
        };

        parser.parse()
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selector::Equality(key, value) => write!(f, "{key}={value}"),
            Selector::Inequality(key, value) => write!(f, "{key}!={value}"),
            Selector::SetInclusion(key, values) => {
                write!(
                    f,
                    "{key} in ({})",
                    values.iter().cloned().collect::<Vec<_>>().join(", ")
                )
            }
            Selector::SetExclusion(key, values) => {
                write!(
                    f,
                    "{key} notin ({})",
                    values.iter().cloned().collect::<Vec<_>>().join(", ")
                )
            }
            Selector::Existence(key) => write!(f, "{key}"),
            Selector::All(selectors) => {
                write!(
                    f,
                    "{}",
                    selectors
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
    }
}

impl Serialize for Selector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = Str::deserialize(deserializer)?;
        Selector::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
struct Lexer<'s> {
    input: &'s str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(Str),
    Equal,
    NotEqual,
    In,
    NotIn,
    Comma,
    LeftParen,
    RightParen,
}

impl Iterator for Lexer<'_> {
    type Item = anyhow::Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input = self.input.trim_start();
        let c = self.input.chars().next()?;

        const PREFIXES: &[(&str, Token)] = &[
            ("notin ", Token::NotIn),
            ("in ", Token::In),
            ("==", Token::Equal),
            ("!=", Token::NotEqual),
            (",", Token::Comma),
            ("(", Token::LeftParen),
            (")", Token::RightParen),
            ("=", Token::Equal),
        ];

        for &(prefix, ref token) in PREFIXES {
            if self.input.starts_with(prefix) {
                self.input = &self.input[prefix.len()..];
                return Some(Ok(token.clone()));
            }
        }

        // Identifiers must start with an alphabetic character.
        if !c.is_alphabetic() {
            return Some(Err(anyhow::anyhow!("unexpected character '{c}'",)));
        }

        let end = self
            .input
            .find(|c: char| c.is_whitespace() || matches!(c, '=' | '!' | ',' | '(' | ')'))
            .unwrap_or(self.input.len());
        let (ident, rest) = self.input.split_at(end);
        self.input = rest;
        Some(Ok(Token::Ident(ident.trim().into())))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn serde() {
        let selectors = [
            "app=nginx",
            "app in (nginx, redis)",
            "app notin (nginx, redis)",
            "app",
            "app!=nginx",
            "app=nginx,env=prod",
            "app!=nginx,env!=prod",
        ];

        for selector in selectors {
            let deserialized = selector.parse::<super::Selector>().unwrap();
            assert_eq!(deserialized.to_string(), selector);
        }

        assert_eq!(
            "app==nginx".parse::<super::Selector>().unwrap(),
            "app=nginx".parse::<super::Selector>().unwrap(),
        );
    }
}
