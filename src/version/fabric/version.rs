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
#[derive(Debug, Clone)]
pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
    pre: Option<PreRelease>,
    build: Option<BuildMetadata>,
}

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

#[derive(Debug, Clone)]
enum Identifier {
    Numeric(usize),
    Textual(String),
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

#[derive(Debug, Clone)]
struct PreRelease {
    idents: Vec<Identifier>,
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

#[derive(Debug, Clone)]
struct BuildMetadata {
    idents: Vec<Identifier>,
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
