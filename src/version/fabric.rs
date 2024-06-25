use std::fmt::Display;
use std::str::FromStr;

pub use super::maven::Version;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    multi::separated_list1,
    sequence::{pair, terminated},
    Finish, IResult, Parser,
};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum_macros::Display;

#[derive(Display, Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
enum Op {
    #[strum(to_string = "=")]
    Exact,
    #[strum(to_string = ">")]
    Greater,
    #[strum(to_string = ">=")]
    GreaterEq,
    #[strum(to_string = "<")]
    Less,
    #[strum(to_string = "<=")]
    LessEq,
    #[strum(to_string = "~")]
    Tilde,
    #[strum(to_string = "^")]
    Caret,
    #[strum(to_string = "*")]
    Wildcard,
}

impl Op {
    fn parse(s: &str) -> IResult<&str, Self> {
        alt((
            tag(">=").map(|_| Self::GreaterEq),
            tag("<=").map(|_| Self::LessEq),
            tag("=").map(|_| Self::Exact),
            tag(">").map(|_| Self::Greater),
            tag("<").map(|_| Self::Less),
            tag("~").map(|_| Self::Tilde),
            tag("^").map(|_| Self::Caret),
            tag("*").map(|_| Self::Wildcard),
        ))
        .parse(s)
    }
}

impl FromStr for Op {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok(("", item)) => Ok(item),
            Ok((rest, item)) => {
                eprintln!("The Op was parsed, but remaining input is left: `{rest}`");
                Ok(item)
            }
            Err(e) => Err(anyhow::anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Comparator {
    operation: Op,
    version: super::FabricVersion,
}

impl Comparator {
    fn parse(s: &str) -> IResult<&str, Self> {
        pair(Op::parse, super::FabricVersion::parse)
            .map(|(op, version)| Self {
                operation: op,
                version,
            })
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
            Err(e) => Err(anyhow::anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{op}{version}",
            op = self.operation,
            version = self.version
        )
    }
}

#[derive(Debug, Clone, PartialEq, DeserializeFromStr, SerializeDisplay, Eq)]
pub struct VersionReq {
    comparators: Vec<Comparator>,
}

impl VersionReq {
    fn parse(s: &str) -> IResult<&str, Self> {
        let separator = terminated(tag(","), space0);
        separated_list1(separator, Comparator::parse)
            .map(|comps| Self { comparators: comps })
            .parse(s)
    }
}

impl FromStr for VersionReq {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok(("", item)) => Ok(item),
            Ok((rest, item)) => {
                eprintln!("The VersionReq was parsed, but remaining input is left: `{rest}`");
                Ok(item)
            }
            Err(e) => Err(anyhow::anyhow!("Error while parsing Comparator: {e}")),
        }
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.comparators
                .iter()
                .map(|comp| comp.to_string())
                .join(",")
        )
    }
}

#[cfg(test)]
mod comparators {
    use super::*;

    #[test]
    fn valid() {
        assert!(Comparator::from_str("=1.1.1").is_ok());
        assert!(Comparator::from_str(">1.1.1").is_ok());
        assert!(Comparator::from_str(">=1.1.1").is_ok());
        assert!(Comparator::from_str("<1.1.1").is_ok());
        assert!(Comparator::from_str("<=1.1.1").is_ok());
        assert!(Comparator::from_str("~1.1.1").is_ok());
        assert!(Comparator::from_str("^1.1.1").is_ok());
        assert!(Comparator::from_str("*1.1.1").is_ok());
    }

    #[test]
    #[should_panic]
    fn invalid() {
        Comparator::from_str("@1.1.1").expect("Should fail because '@' is not a valid comparator");
    }
}

#[cfg(test)]
mod multiversion {
    use super::*;

    #[test]
    #[should_panic]
    fn empty() {
        VersionReq::from_str("").expect("Should fail because empty string is not a valid version");
    }

    #[test]
    fn basic() -> anyhow::Result<()> {
        VersionReq::from_str(">=1.2.3, <1.8.0")?;

        Ok(())
    }
}
