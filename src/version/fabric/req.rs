use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0},
    combinator::opt,
    multi::separated_list1,
    sequence::{preceded, terminated},
    IResult, Parser,
};
use strum_macros::Display;

use super::utils::*;
use super::version::PreRelease;

#[derive(Display, Debug, Clone)]
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
        major
            .and(opt(minor))
            .and(opt(patch))
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

#[derive(Display, Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct VersionReq {
    ops: Vec<Op>,
}

impl VersionReq {
    fn parse(input: &str) -> IResult<&str, Self> {
        let separator = terminated(char(','), space0);
        separated_list1(separator, Op::parse)
            .map(|ops| Self { ops })
            .parse(input)
    }
}
