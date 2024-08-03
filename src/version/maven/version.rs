use anyhow::anyhow;
use itertools::Itertools;
use nom::{
    character::complete::{alpha1, digit1, one_of},
    combinator::{all_consuming, map_res},
    multi::separated_list1,
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay, Hash)]
pub struct Version {
    items: Vec<VersionItem>,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.items.iter().map(|item| item.to_string()).join(".")
        )
    }
}

impl Version {
    pub(super) fn parse(s: &str) -> IResult<&str, Self> {
        separated_list1(one_of(".-"), VersionItem::parse)
            .map(|items| Self { items })
            .parse(s)
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(Self::parse).parse(s).finish() {
            Ok((_, item)) => Ok(item),
            Err(e) => Err(anyhow!("Error while parsing Version: {e}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum VersionItem {
    Numeric(usize),
    /// e.g. snapshot or beta
    Textual(String),
}

impl std::fmt::Display for VersionItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionItem::Numeric(number) => write!(f, "{number}"),
            VersionItem::Textual(text) => write!(f, "{text}"),
        }
    }
}

impl VersionItem {
    fn parse(s: &str) -> IResult<&str, Self> {
        map_res(digit1, str::parse::<usize>)
            .map(Self::Numeric)
            .or(alpha1
                .map(|value: &str| value.to_lowercase())
                .map(Self::Textual))
            .parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(items: Vec<VersionItem>) -> Version {
        Version { items }
    }

    #[test]
    fn basic() {
        assert!(Version::from_str("1.2.3.4.5")
            .is_ok_and(|ver| ver == version((1..=5).map(VersionItem::Numeric).collect())));

        assert!(Version::from_str("1.20.1").is_ok_and(
            |ver| ver == version([1, 20, 1].into_iter().map(VersionItem::Numeric).collect())
        ));

        assert!(Version::from_str("1.20-SNAPSHOT").is_ok_and(|ver| ver
            == version(vec![
                VersionItem::Numeric(1),
                VersionItem::Numeric(20),
                VersionItem::Textual(String::from("snapshot"))
            ])));

        assert!(Version::parse(".10").is_err());
    }

    #[test]
    fn version_item_numeric() {
        assert_eq!(
            VersionItem::parse("123"),
            Ok(("", VersionItem::Numeric(123)))
        );

        assert_eq!(VersionItem::parse("003"), Ok(("", VersionItem::Numeric(3))));
    }

    #[test]
    fn version_item_textual() {
        assert_eq!(
            VersionItem::parse("SNAPSHOT"),
            Ok(("", VersionItem::Textual(String::from("snapshot"))))
        );
    }
}
