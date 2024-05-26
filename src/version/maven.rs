use anyhow::anyhow;
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, one_of},
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, terminated},
    Finish, IResult, Parser,
};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Comparator {
    Exact(Version),
    Greater(Version),
    GreaterEq(Version),
    Less(Version),
    LessEq(Version),
    BetweenInclusive(Version, Version),
    BetweenUnInclusive(Version, Version),
}

impl Comparator {
    fn parse(s: &str) -> IResult<&str, Self> {
        macro_rules! single {
            ($op:literal x $cl:literal) => {
                delimited(tag($op), Version::parse, tag($cl))
            };
        }

        macro_rules! double {
            ($op:literal x y $cl:literal) => {
                preceded(
                    tag($op),
                    terminated(
                        separated_pair(Version::parse, tag(","), Version::parse),
                        tag($cl),
                    ),
                )
            };
        }

        single!("(," x "]")
            .map(Self::LessEq)
            .or(single!("(," x ")").map(Self::Less))
            .or(single!("[" x "]").map(Self::Exact))
            .or(single!("[" x ",)").map(Self::GreaterEq))
            .or(single!("(" x ",)").map(Self::Greater))
            .or(double!("[" x y "]").map(|(x, y)| Self::BetweenInclusive(x, y)))
            .or(double!("(" x y ")").map(|(x, y)| Self::BetweenUnInclusive(x, y)))
            .or(map(Version::parse, Self::GreaterEq))
            .parse(s)
    }
}

impl FromStr for Comparator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok(("", item)) => Ok(item),
            Ok((rest, item)) => {
                eprintln!("The Comparator was parsed, but remaining input is left: `{rest}`");
                Ok(item)
            }
            Err(e) => Err(anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VersionRange {
    comparators: Vec<Comparator>,
}

impl VersionRange {
    fn parse(s: &str) -> IResult<&str, Self> {
        separated_list1(tag(","), Comparator::parse)
            .map(|comparators| Self { comparators })
            .parse(s)
    }
}

impl FromStr for VersionRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok(("", item)) => Ok(item),
            Ok((rest, item)) => {
                eprintln!("The VersionRange was parsed, but remaining input is left: `{rest}`");
                Ok(item)
            }
            Err(e) => Err(anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionItem {
    Numeric(usize),
    Textual(String),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    items: Vec<VersionItem>,
}

impl Version {
    fn parse(s: &str) -> IResult<&str, Self> {
        separated_list1(one_of(".-"), VersionItem::parse)
            .map(|items| Self { items })
            .parse(s)
    }

    fn new(items: Vec<VersionItem>) -> Self {
        Self { items }
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok(("", item)) => Ok(item),
            Ok((rest, item)) => {
                eprintln!("The Version was parsed, but remaining input is left: `{rest}`");
                Ok(item)
            }
            Err(e) => Err(anyhow!("Error while parsing Version: {e}")),
        }
    }
}

#[cfg(test)]
mod comparators {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(
            Comparator::from_str("1.0").ok(),
            Some(Comparator::GreaterEq(Version::from_str("1.0").unwrap()))
        );

        assert!(Comparator::from_str("(,1.0]").is_ok());
        assert!(Comparator::from_str("(,1.0)").is_ok());
        assert!(Comparator::from_str("[1.0]").is_ok());
        assert!(Comparator::from_str("[1.0]").is_ok());
        assert!(Comparator::from_str("[1.0,)").is_ok());
        assert!(Comparator::from_str("(1.0,)").is_ok());
        assert!(Comparator::from_str("(1.0,2.0)")
            .is_ok_and(|item| matches!(item, Comparator::BetweenUnInclusive(_, _))));
        assert!(Comparator::from_str("[1.0,2.0]")
            .is_ok_and(|item| matches!(item, Comparator::BetweenInclusive(_, _))));
    }

    #[test]
    fn multiple() {
        assert!(VersionRange::from_str("(,1.0],[1.2,)").is_ok());
        assert!(VersionRange::from_str("(,1.1),(1.1,)").is_ok());
    }
}

#[cfg(test)]
mod versions {
    use super::*;

    #[test]
    fn version() {
        assert_eq!(
            Version::parse("1.2.3.4.5"),
            Ok((
                "",
                Version::new((1..=5).map(VersionItem::Numeric).collect())
            ))
        );

        assert_eq!(
            Version::parse("1.20.1"),
            Ok((
                "",
                Version::new([1, 20, 1].into_iter().map(VersionItem::Numeric).collect())
            ))
        );

        assert_eq!(
            Version::parse("1.20-SNAPSHOT"),
            Ok((
                "",
                Version::new(vec![
                    VersionItem::Numeric(1),
                    VersionItem::Numeric(20),
                    VersionItem::Textual(String::from("snapshot"))
                ])
            ))
        );

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
