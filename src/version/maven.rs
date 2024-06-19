use super::VersionItem;

use anyhow::anyhow;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::one_of,
    combinator::opt,
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, terminated},
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Comparator {
    Minimum(Version),
    Exact(Version),
    Pair {
        left: ComparatorHalf,
        right: ComparatorHalf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ComparatorHalf {
    Inclusive(Option<Version>),
    Uninclusive(Option<Version>),
}

impl ComparatorHalf {
    fn parse_left(s: &str) -> IResult<&str, Self> {
        preceded(tag("("), opt(Version::parse))
            .map(Self::Uninclusive)
            .or(preceded(tag("["), opt(Version::parse)).map(Self::Inclusive))
            .parse(s)
    }

    fn parse_right(s: &str) -> IResult<&str, Self> {
        terminated(opt(Version::parse), tag(")"))
            .map(Self::Uninclusive)
            .or(terminated(opt(Version::parse), tag("]")).map(Self::Inclusive))
            .parse(s)
    }

    fn parse(s: &str) -> IResult<&str, Self> {
        Self::parse_left.or(Self::parse_right).parse(s)
    }
}

impl FromStr for ComparatorHalf {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok(("", item)) => Ok(item),
            Ok((rest, item)) => {
                eprintln!("The ComparatorHalf was parsed, but remaining input is left: `{rest}`");
                Ok(item)
            }
            Err(e) => Err(anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

impl std::fmt::Display for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparator::Minimum(version) => write!(f, "{version}"),
            Comparator::Exact(version) => write!(f, "[{version}]"),
            Comparator::Pair { left, right } => {
                match left {
                    ComparatorHalf::Inclusive(maybe_version) => match maybe_version {
                        Some(version) => write!(f, "[{version}"),
                        None => write!(f, "["),
                    },
                    ComparatorHalf::Uninclusive(maybe_version) => match maybe_version {
                        Some(version) => write!(f, "({version}"),
                        None => write!(f, "("),
                    },
                }?;

                match right {
                    ComparatorHalf::Inclusive(maybe_version) => match maybe_version {
                        Some(version) => write!(f, "{version}]"),
                        None => write!(f, "]"),
                    },
                    ComparatorHalf::Uninclusive(maybe_version) => match maybe_version {
                        Some(version) => write!(f, "{version})"),
                        None => write!(f, ")"),
                    },
                }
            }
        }
    }
}

impl Comparator {
    fn parse(s: &str) -> IResult<&str, Self> {
        // 1. Try just version
        // 2. Try version in square brackets
        // 3. Split by comma and process
        Version::parse
            .map(Self::Minimum)
            .or(delimited(tag("["), Version::parse, tag("]")).map(Self::Exact))
            .or(separated_pair(
                ComparatorHalf::parse_left,
                tag(","),
                ComparatorHalf::parse_right,
            )
            .map(|(left, right)| Self::Pair { left, right }))
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

#[derive(Debug, Clone, DeserializeFromStr, SerializeDisplay, PartialEq, Eq)]
pub struct VersionRange {
    comparators: Vec<Comparator>,
}

impl std::fmt::Display for VersionRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.comparators.len() == 1 {
            write!(f, "{}", self.comparators.first().unwrap())
        } else {
            write!(
                f,
                "{}",
                self.comparators
                    .iter()
                    .map(|comparator| comparator.to_string())
                    .join(",")
            )
        }
    }
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
    fn parse(s: &str) -> IResult<&str, Self> {
        separated_list1(one_of(".-"), VersionItem::parse)
            .map(|items| Self { items })
            .parse(s)
    }

    pub fn new(items: Vec<VersionItem>) -> Self {
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
mod comparator_halves {
    use super::*;

    #[test]
    fn inclusive() {
        assert!(ComparatorHalf::from_str("[1.20")
            .is_ok_and(|version| matches!(version, ComparatorHalf::Inclusive(_))));
        assert!(ComparatorHalf::from_str("1.20]")
            .is_ok_and(|version| matches!(version, ComparatorHalf::Inclusive(_))));
    }

    #[test]
    fn uninclusive() {
        assert!(ComparatorHalf::from_str("(1.20")
            .is_ok_and(|version| matches!(version, ComparatorHalf::Uninclusive(_))));
        assert!(ComparatorHalf::from_str("1.20)")
            .is_ok_and(|version| matches!(version, ComparatorHalf::Uninclusive(_))));
    }
}

#[cfg(test)]
mod comparators {
    use super::*;

    #[test]
    fn no_commas() {
        assert!(VersionRange::from_str("1.0").is_ok());
        assert!(VersionRange::from_str("[1.0]").is_ok());
    }

    #[test]
    fn commas_halves() {
        assert!(VersionRange::from_str("(,1.0]").is_ok());
        assert!(VersionRange::from_str("(,1.0)").is_ok());
        assert!(VersionRange::from_str("[1.0,)").is_ok());
        assert!(VersionRange::from_str("(1.0,)").is_ok());
    }

    #[test]
    fn commas_double() {
        assert!(VersionRange::from_str("(1.0,2.0)").is_ok());
        assert!(VersionRange::from_str("[1.0,2.0]").is_ok());
    }

    #[test]
    fn mixed() {
        assert!(VersionRange::from_str("[1.20,1.21)").is_ok());
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
