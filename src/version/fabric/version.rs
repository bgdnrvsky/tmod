use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    character::complete::{char, digit1, one_of},
    combinator::map_res,
    multi::{many1, separated_list1},
    sequence::{separated_pair, tuple},
    Finish, IResult, Parser,
};

/// Custom implementation of semver Version.
/// Needed since the `semver` crate isn't flexible enough
#[derive(Debug, Clone, Hash)]
pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
    pre: Option<PreRelease>,
    build: Option<BuildMetadata>,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(pre_release) = &self.pre {
            write!(f, "-{}", pre_release)?;
        }

        if let Some(build_meta) = &self.build {
            write!(f, "+{}", build_meta)?;
        }

        Ok(())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major.eq(&other.major)
            && self.minor.eq(&other.minor)
            && self.patch.eq(&other.patch)
            && self.pre.eq(&other.pre)
    }
}

impl Eq for Version {}

impl Version {
    fn parse(input: &str) -> IResult<&str, Self> {
        fn simple_semver(input: &str) -> IResult<&str, Version> {
            version_core
                .map(|(major, minor, patch)| Version {
                    major,
                    minor,
                    patch,
                    pre: None,
                    build: None,
                })
                .parse(input)
        }

        fn core_and_release(input: &str) -> IResult<&str, Version> {
            separated_pair(version_core, char('-'), PreRelease::parse)
                .map(|((major, minor, patch), pre)| Version {
                    major,
                    minor,
                    patch,
                    pre: Some(pre),
                    build: None,
                })
                .parse(input)
        }

        fn core_and_build(input: &str) -> IResult<&str, Version> {
            separated_pair(version_core, char('+'), BuildMetadata::parse)
                .map(|((major, minor, patch), build)| Version {
                    major,
                    minor,
                    patch,
                    pre: None,
                    build: Some(build),
                })
                .parse(input)
        }

        fn full_semver(input: &str) -> IResult<&str, Version> {
            tuple((
                version_core,
                char('-'),
                PreRelease::parse,
                char('+'),
                BuildMetadata::parse,
            ))
            .map(|((major, minor, patch), _, pre, _, build)| Version {
                major,
                minor,
                patch,
                pre: Some(pre),
                build: Some(build),
            })
            .parse(input)
        }

        alt((full_semver, simple_semver, core_and_release, core_and_build)).parse(input)
    }
}

impl std::str::FromStr for Version {
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

#[derive(Debug, Clone, PartialEq, Hash, Ord, Eq)]
enum Identifier {
    Numeric(usize),
    Textual(String),
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Numeric(num) => write!(f, "{num}"),
            Identifier::Textual(text) => write!(f, "{text}"),
        }
    }
}

impl PartialOrd for Identifier {
    // 1. Identifiers with letters or hyphens are compared lexically in ASCII sort order.
    // 2. Numeric identifiers always have lower precedence than non-numeric identifiers.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Identifier::Numeric(a), Identifier::Numeric(b)) => a.partial_cmp(b),
            (Identifier::Numeric(_), Identifier::Textual(_)) => Some(std::cmp::Ordering::Less),
            (Identifier::Textual(_), Identifier::Numeric(_)) => Some(std::cmp::Ordering::Greater),
            (Identifier::Textual(a), Identifier::Textual(b)) => a.partial_cmp(b),
        }
    }
}

impl Identifier {
    fn parse(s: &str) -> IResult<&str, Self> {
        let numeric = map_res(digit1, str::parse::<usize>).map(Self::Numeric);
        let textual = many1(one_of("abcdefghijklmnopqrstuvwxyz-"))
            .map(|chars| String::from_iter(chars))
            .map(Self::Textual);

        numeric.or(textual).parse(s)
    }
}

impl std::str::FromStr for Identifier {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PreRelease {
    idents: Vec<Identifier>,
}

impl Display for PreRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.idents.iter().map(ToString::to_string).join(".")
        )
    }
}

impl PreRelease {
    fn parse(input: &str) -> IResult<&str, Self> {
        separated_list1(char('.'), Identifier::parse)
            .map(|idents| Self { idents })
            .parse(input)
    }
}

impl std::str::FromStr for PreRelease {
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

#[derive(Debug, Clone, Hash)]
struct BuildMetadata {
    idents: Vec<Identifier>,
}

impl Display for BuildMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.idents.iter().map(ToString::to_string).join(".")
        )
    }
}

impl BuildMetadata {
    fn parse(input: &str) -> IResult<&str, Self> {
        separated_list1(char('.'), Identifier::parse)
            .map(|idents| Self { idents })
            .parse(input)
    }
}

impl std::str::FromStr for BuildMetadata {
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

fn version_core(input: &str) -> IResult<&str, (usize, usize, usize)> {
    tuple((major, char('.'), minor, char('.'), patch))
        .map(|(maj, _, min, _, pat)| (maj, min, pat))
        .parse(input)
}

fn major(input: &str) -> IResult<&str, usize> {
    decimal(input)
}

fn minor(input: &str) -> IResult<&str, usize> {
    decimal(input)
}

fn patch(input: &str) -> IResult<&str, usize> {
    decimal(input)
}

fn decimal(input: &str) -> IResult<&str, usize> {
    map_res(digit1, str::parse::<usize>).parse(input)
}
