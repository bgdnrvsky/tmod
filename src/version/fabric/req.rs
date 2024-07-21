use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::opt,
    multi::separated_list1,
    sequence::{preceded, terminated},
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum_macros::Display;

use super::utils::*;
use super::version::PreRelease;

#[derive(Display, Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
enum VersionPart {
    #[strum(to_string = "{major}")]
    Single { major: usize },
    #[strum(to_string = "{major}.{minor}")]
    Double { major: usize, minor: usize },
    #[strum(to_string = "{major}.{minor}.{patch}")]
    Triple {
        major: usize,
        minor: usize,
        patch: usize,
    },
    #[strum(to_string = "{major}.{minor}.{patch}-{pre}")]
    Full {
        major: usize,
        minor: usize,
        patch: usize,
        pre: PreRelease,
    },
}

impl VersionPart {
    fn parse(input: &str) -> IResult<&str, Self> {
        // TODO: Remake it
        decimal(false)
            .and(opt(decimal(false)))
            .and(opt(decimal(false)))
            .and(opt(PreRelease::parse))
            .map(|comb| -> Self {
                match comb {
                    (((major, None), None), None) => Self::Single { major },
                    (((major, Some(minor)), None), None) => Self::Double { major, minor },
                    (((major, Some(minor)), Some(patch)), None) => Self::Triple {
                        major,
                        minor,
                        patch,
                    },
                    (((major, Some(minor)), Some(patch)), Some(pre)) => Self::Full {
                        major,
                        minor,
                        patch,
                        pre,
                    },
                    _ => unreachable!(),
                }
            })
            .parse(input)
    }
}

impl std::str::FromStr for VersionPart {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok((_, version)) => Ok(version),
            Err(nom::error::Error { input, code }) => Err(Self::Err {
                input: input.to_string(),
                code,
            }),
        }
    }
}

#[derive(Display, Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
enum Op {
    #[strum(to_string = "={0}")]
    Exact(VersionPart),
    #[strum(to_string = ">{0}")]
    Greater(VersionPart),
    #[strum(to_string = ">={0}")]
    GreaterEq(VersionPart),
    #[strum(to_string = "<{0}")]
    Less(VersionPart),
    #[strum(to_string = "<={0}")]
    LessEq(VersionPart),
    #[strum(to_string = "~{0}")]
    Tilde(VersionPart),
    #[strum(to_string = "^{0}")]
    Caret(VersionPart),
    #[strum(to_string = "*")]
    Wildcard,
}

impl Op {
    fn parse(input: &str) -> IResult<&str, Self> {
        let op = |prefix| preceded(tag(prefix), VersionPart::parse);
        alt((
            op(">=").map(Self::GreaterEq),
            op("<=").map(Self::LessEq),
            op("=").map(Self::Exact),
            op(">").map(Self::Greater),
            op("<").map(Self::Less),
            op("~").map(Self::Tilde),
            op("^").map(Self::Caret),
            char('*').map(|_| Self::Wildcard),
        ))
        .parse(input)
    }
}

impl std::str::FromStr for Op {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok((_, version)) => Ok(version),
            Err(nom::error::Error { input, code }) => Err(Self::Err {
                input: input.to_string(),
                code,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct VersionReq {
    ops: Vec<Op>,
}

impl VersionReq {
    pub fn any() -> Self {
        Self {
            ops: vec![Op::Wildcard],
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        let separator = terminated(char(','), space0);
        separated_list1(separator, Op::parse)
            .map(|ops| Self { ops })
            .parse(input)
    }
}

impl std::str::FromStr for VersionReq {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s).finish() {
            Ok((_, version)) => Ok(version),
            Err(nom::error::Error { input, code }) => Err(Self::Err {
                input: input.to_string(),
                code,
            }),
        }
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ops.iter().map(ToString::to_string).join(", "))
    }
}
