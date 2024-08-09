use std::{fmt::Display, str::FromStr};

use itertools::Itertools;
use nom::{
    character::complete::{char, space0},
    combinator::{all_consuming, opt},
    multi::separated_list1,
    sequence::{delimited, preceded, separated_pair, terminated},
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::version::Version;

#[derive(Debug, Clone, DeserializeFromStr, SerializeDisplay, PartialEq, Eq)]
pub struct VersionRange {
    comparators: Vec<Comparator>,
}

impl Display for VersionRange {
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
        separated_list1(terminated(char(','), space0), Comparator::parse)
            .map(|comparators| Self { comparators })
            .parse(s)
    }
}

impl FromStr for VersionRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(Self::parse).parse(s).finish() {
            Ok((_, item)) => Ok(item),
            Err(e) => Err(anyhow::anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

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
    fn get(&self) -> Option<&Version> {
        match self {
            Self::Inclusive(version) | Self::Uninclusive(version) => {
                version.as_ref()
            }
        }
    }

    fn parse_left(s: &str) -> IResult<&str, Self> {
        preceded(char('('), opt(Version::parse))
            .map(Self::Uninclusive)
            .or(preceded(char('['), opt(Version::parse)).map(Self::Inclusive))
            .parse(s)
    }

    fn parse_right(s: &str) -> IResult<&str, Self> {
        terminated(opt(Version::parse), char(')'))
            .map(Self::Uninclusive)
            .or(terminated(opt(Version::parse), char(']')).map(Self::Inclusive))
            .parse(s)
    }

    fn parse(s: &str) -> IResult<&str, Self> {
        Self::parse_left.or(Self::parse_right).parse(s)
    }

    /// Returns `true` if the comparator half is [`Inclusive`].
    ///
    /// [`Inclusive`]: ComparatorHalf::Inclusive
    fn is_inclusive(&self) -> bool {
        matches!(self, Self::Inclusive(..))
    }

    /// Returns `true` if the comparator half is [`Uninclusive`].
    ///
    /// [`Uninclusive`]: ComparatorHalf::Uninclusive
    fn is_uninclusive(&self) -> bool {
        matches!(self, Self::Uninclusive(..))
    }
}

impl FromStr for ComparatorHalf {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(Self::parse).parse(s).finish() {
            Ok((_, item)) => Ok(item),
            Err(e) => Err(anyhow::anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Comparator::Minimum(version) => write!(f, "{version}"),
            Comparator::Exact(version) => write!(f, "[{version}]"),
            Comparator::Pair { left, right } => {
                let left_bracket: char = if left.is_inclusive() { '[' } else { '(' };
                let right_bracket: char = if right.is_uninclusive() { ')' } else { ']' };

                let (left_version, right_version) = (left.get(), right.get());

                if left_version.is_none() && right_version.is_none() {
                    panic!("Both sides of comparator can't be None");
                }

                write!(
                    f,
                    "{left_bracket}{left_version},{right_version}{right_bracket}",
                    left_version = left_version.map(Version::to_string).unwrap_or_default(),
                    right_version = right_version.map(Version::to_string).unwrap_or_default(),
                )
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
            .or(delimited(char('['), Version::parse, char(']')).map(Self::Exact))
            .or(separated_pair(
                ComparatorHalf::parse_left,
                terminated(char(','), space0),
                ComparatorHalf::parse_right,
            )
            .map(|(left, right)| Self::Pair { left, right }))
            .parse(s)
    }
}

impl FromStr for Comparator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(Self::parse).parse(s).finish() {
            Ok((_, item)) => Ok(item),
            Err(e) => Err(anyhow::anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    mod comparator_halves {
        use super::*;

        #[test]
        fn inclusive() -> anyhow::Result<()> {
            let left = ComparatorHalf::from_str("[1.20")?;
            assert!(left.is_inclusive());

            let right = ComparatorHalf::from_str("1.20]")?;
            assert!(right.is_inclusive());

            Ok(())
        }

        #[test]
        fn uninclusive() -> anyhow::Result<()> {
            let left = ComparatorHalf::from_str("(1.20")?;
            assert!(left.is_uninclusive());

            let right = ComparatorHalf::from_str("1.20)")?;
            assert!(right.is_uninclusive());

            Ok(())
        }
    }

    #[cfg(test)]
    mod comparators {
        use super::*;

        #[test]
        fn no_commas() -> anyhow::Result<()> {
            VersionRange::from_str("1.0")?;
            VersionRange::from_str("[1.0]")?;

            Ok(())
        }

        #[test]
        fn commas_halves() -> anyhow::Result<()> {
            VersionRange::from_str("(,1.0]")?;
            VersionRange::from_str("(,1.0)")?;
            VersionRange::from_str("[1.0,)")?;
            VersionRange::from_str("(1.0,)")?;

            Ok(())
        }

        #[test]
        fn commas_double() -> anyhow::Result<()> {
            VersionRange::from_str("(1.0,2.0)")?;
            VersionRange::from_str("[1.0,2.0]")?;

            Ok(())
        }

        #[test]
        fn double_comma_spaces() -> anyhow::Result<()> {
            VersionRange::from_str("(1.0, 2.0)")?;
            VersionRange::from_str("[1.0,            2.0]")?;

            Ok(())
        }

        #[test]
        fn mixed() -> anyhow::Result<()> {
            VersionRange::from_str("[1.20,1.21)")?;

            Ok(())
        }

        #[test]
        fn multiple() -> anyhow::Result<()> {
            VersionRange::from_str("(,1.0],[1.2,)")?;
            VersionRange::from_str("(,1.1),(1.1,)")?;

            Ok(())
        }
    }
}
