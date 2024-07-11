use std::fmt::Display;
use std::str::FromStr;

pub use super::maven::Version;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::space0,
    multi::separated_list1,
    sequence::{preceded, terminated},
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum_macros::Display;

#[derive(Display, Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
enum Op {
    #[strum(to_string = "={0}")]
    Exact(super::FabricVersion),
    #[strum(to_string = ">{0}")]
    Greater(super::FabricVersion),
    #[strum(to_string = ">={0}")]
    GreaterEq(super::FabricVersion),
    #[strum(to_string = "<{0}")]
    Less(super::FabricVersion),
    #[strum(to_string = "<={0}")]
    LessEq(super::FabricVersion),
    #[strum(to_string = "~{0}")]
    Tilde(super::FabricVersion),
    #[strum(to_string = "^{0}")]
    Caret(super::FabricVersion),
    #[strum(to_string = "*")]
    Wildcard,
}

impl Op {
    fn parse(s: &str) -> IResult<&str, Self> {
        let paired_op = |input| preceded(tag(input), super::FabricVersion::parse);
        alt((
            paired_op(">=").map(Self::GreaterEq),
            paired_op("<=").map(Self::LessEq),
            paired_op("=").map(Self::Exact),
            paired_op(">").map(Self::Greater),
            paired_op("<").map(Self::Less),
            paired_op("~").map(Self::Tilde),
            paired_op("^").map(Self::Caret),
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
            Err(e) => Err(anyhow::anyhow!("Error while parsing Operation: {e}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, DeserializeFromStr, SerializeDisplay, Eq)]
pub struct VersionReq {
    operations: Vec<Op>,
}

impl VersionReq {
    fn parse(s: &str) -> IResult<&str, Self> {
        let separator = terminated(tag(","), space0);
        separated_list1(separator, Op::parse)
            .map(|operations| Self { operations })
            .parse(s)
    }

    pub fn any() -> VersionReq {
        Self {
            operations: vec![Op::Wildcard],
        }
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
            Err(e) => Err(anyhow::anyhow!("Error while parsing VersionReq: {e}")),
        }
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.operations.iter().map(|op| op.to_string()).join(",")
        )
    }
}

#[cfg(test)]
mod operations {
    use super::*;

    #[test]
    fn valid() {
        assert!(Op::from_str("=1.1.1").is_ok());
        assert!(Op::from_str(">1.1.1").is_ok());
        assert!(Op::from_str(">=1.1.1").is_ok());
        assert!(Op::from_str("<1.1.1").is_ok());
        assert!(Op::from_str("<=1.1.1").is_ok());
        assert!(Op::from_str("~1.1.1").is_ok());
        assert!(Op::from_str("^1.1.1").is_ok());
        assert!(Op::from_str("*1.1.1").is_ok());
    }

    #[test]
    #[should_panic]
    fn invalid() {
        Op::from_str("@1.1.1").expect("Should fail because '@' is not a valid operation");
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

    #[test]
    fn any() -> anyhow::Result<()> {
        VersionReq::from_str("*")?;
        VersionReq::from_str(">=1.2.3, *")?;

        Ok(())
    }
}
