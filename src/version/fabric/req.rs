use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, satisfy, space0},
    combinator::{all_consuming, cond, opt},
    multi::separated_list1,
    sequence::{delimited, preceded},
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::version::{BuildMetadata, PreRelease};
use super::{utils::decimal, Version};

#[derive(Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct VersionReq {
    comparators: Vec<Comparator>,
}

impl VersionReq {
    pub fn any() -> Self {
        Self {
            comparators: Vec::new(),
        }
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        let (rest, wildcard) = preceded(space0, opt(Op::parse_wildcard)).parse(input)?;

        if wildcard.is_some() {
            if rest.is_empty() {
                return Ok((rest, Self::any()));
            } else {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Satisfy,
                )));
            }
        }

        let separator = delimited(space0, char(','), space0);
        separated_list1(separator, Comparator::parse)
            .map(|comparators| Self { comparators })
            .parse(input)
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.comparators
            .iter()
            .all(|comparator| comparator.matches(version))
            && {
                // If a version has a prerelease or build metadata tag then it
                // will only be allowed to satisfy req if at least one comparator with the
                // same major.minor.patch also has a prerelease or build metadata tag.

                let has_prerelease = version.pre.is_some();
                let has_buildmetadata = version.build.is_some();

                if !has_prerelease && !has_buildmetadata {
                    return true;
                }

                self.comparators.iter().any(|comparator| {
                    (comparator.is_compatible_prerelease(version) || !has_prerelease)
                        && (comparator.is_compatible_buildmeta(version) || !has_buildmetadata)
                })
            }
    }
}

impl Default for VersionReq {
    fn default() -> Self {
        Self::any()
    }
}

impl std::str::FromStr for VersionReq {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(Self::parse).parse(s).finish() {
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
        if self.comparators.is_empty() {
            return write!(f, "*");
        }

        write!(
            f,
            "{}",
            self.comparators.iter().map(ToString::to_string).join(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Comparator {
    operation: Op,
    major: u64,
    minor: Option<u64>,
    patch: Option<u64>,
    pre: Option<PreRelease>,
    build: Option<BuildMetadata>,
}

impl Comparator {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (rest, mut operation) = opt(Op::parse)
            .map(|maybe_op| maybe_op.unwrap_or_default())
            .parse(input)?;

        let (rest, major) = decimal(false).parse(rest)?;

        let (rest, minor) = cond(rest.starts_with('.'), VersionPart::parse).parse(rest)?;

        if minor.as_ref().is_some_and(VersionPart::is_wildcard) {
            operation = Op::Wildcard;
        }

        let (rest, patch) =
            cond(rest.starts_with('.') && minor.is_some(), VersionPart::parse).parse(rest)?;

        if patch.as_ref().is_some_and(VersionPart::is_wildcard) {
            operation = Op::Wildcard;
        }

        let (rest, pre) = cond(rest.starts_with('-'), PreRelease::parse).parse(rest)?;

        let (rest, build) = cond(rest.starts_with('+'), BuildMetadata::parse).parse(rest)?;

        if patch.is_some() {
            if minor.is_none() {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Satisfy,
                )));
            }

            if patch.as_ref().is_some_and(VersionPart::is_numeric)
                && minor.as_ref().is_some_and(VersionPart::is_wildcard)
            {
                return Err(nom::Err::Failure(nom::error::Error::new(
                    rest,
                    nom::error::ErrorKind::Satisfy,
                )));
            }
        }

        Ok((
            rest,
            Self {
                operation,
                major,
                minor: minor.and_then(VersionPart::resolve),
                patch: patch.and_then(VersionPart::resolve),
                pre,
                build,
            },
        ))
    }

    fn is_compatible_version_base(&self, version: &Version) -> bool {
        self.major == version.major
            && self.minor == Some(version.minor)
            && self.patch == Some(version.patch)
    }

    fn is_compatible_prerelease(&self, version: &Version) -> bool {
        self.is_compatible_version_base(version) && self.pre.is_some()
    }

    fn is_compatible_buildmeta(&self, version: &Version) -> bool {
        self.is_compatible_version_base(version) && self.build.is_some()
    }

    fn matches_exact(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        if let Some(minor) = self.minor {
            if version.minor != minor {
                return false;
            }
        }

        if let Some(patch) = self.patch {
            if version.patch != patch {
                return false;
            }
        }

        let match_pre = match (self.pre.as_ref(), version.pre.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => false,
            (Some(a), Some(b)) => a == b,
        };

        let match_build = match (self.build.as_ref(), version.build.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(_), Some(_)) => true,
        };

        match_pre && match_build
    }

    fn matches_greater(&self, version: &Version) -> bool {
        if version.major != self.major {
            return version.major > self.major;
        }

        match self.minor {
            None => return false,
            Some(minor) => {
                if version.minor != minor {
                    return version.minor > minor;
                }
            }
        }

        match self.patch {
            None => return false,
            Some(patch) => {
                if version.patch != patch {
                    return version.patch > patch;
                }
            }
        }

        let match_pre = match (self.pre.as_ref(), version.pre.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(a), Some(b)) => b > a,
        };

        let match_build = match (self.build.as_ref(), version.build.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(_), Some(_)) => true,
        };

        match_pre && match_build
    }

    fn matches_less(&self, version: &Version) -> bool {
        if version.major != self.major {
            return version.major < self.major;
        }

        match self.minor {
            None => return false,
            Some(minor) => {
                if version.minor != minor {
                    return version.minor < minor;
                }
            }
        }

        match self.patch {
            None => return false,
            Some(patch) => {
                if version.patch != patch {
                    return version.patch < patch;
                }
            }
        }

        let match_pre = match (self.pre.as_ref(), version.pre.as_ref()) {
            (None, None) => false,
            (None, Some(_)) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => b < a,
        };

        let match_build = match (self.build.as_ref(), version.build.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(_), Some(_)) => true,
        };

        match_pre && match_build
    }

    fn matches_tilde(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        if let Some(minor) = self.minor {
            if version.minor != minor {
                return false;
            }
        }

        if let Some(patch) = self.patch {
            if version.patch != patch {
                return version.patch > patch;
            }
        }

        let match_pre = match (self.pre.as_ref(), version.pre.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(a), Some(b)) => b >= a,
        };

        let match_build = match (self.build.as_ref(), version.build.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(_), Some(_)) => true,
        };

        match_pre && match_build
    }

    fn matches_caret(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        let minor = match self.minor {
            None => return true,
            Some(minor) => minor,
        };

        let patch = match self.patch {
            None => {
                if self.major > 0 {
                    return version.minor >= minor;
                } else {
                    return version.minor == minor;
                }
            }
            Some(patch) => patch,
        };

        if self.major > 0 {
            if version.minor != minor {
                return version.minor > minor;
            } else if version.patch != patch {
                return version.patch > patch;
            }
        } else if minor > 0 {
            if version.minor != minor {
                return false;
            } else if version.patch != patch {
                return version.patch > patch;
            }
        } else if version.minor != minor || version.patch != patch {
            return false;
        }

        let match_pre = match (self.pre.as_ref(), version.pre.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(a), Some(b)) => b >= a,
        };

        let match_build = match (self.build.as_ref(), version.build.as_ref()) {
            (None, None) => true,
            (None, Some(_)) => false,
            (Some(_), None) => true,
            (Some(_), Some(_)) => true,
        };

        match_pre && match_build
    }

    fn matches(&self, ver: &Version) -> bool {
        match self.operation {
            Op::Exact | Op::Wildcard => self.matches_exact(ver),
            Op::Greater => self.matches_greater(ver),
            Op::GreaterEq => self.matches_exact(ver) || self.matches_greater(ver),
            Op::Less => self.matches_less(ver),
            Op::LessEq => self.matches_exact(ver) || self.matches_less(ver),
            Op::Tilde => self.matches_tilde(ver),
            Op::Caret => self.matches_caret(ver),
        }
    }
}

impl std::str::FromStr for Comparator {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match all_consuming(Self::parse).parse(s).finish() {
            Ok((_, version)) => Ok(version),
            Err(nom::error::Error { input, code }) => Err(Self::Err {
                input: input.to_string(),
                code,
            }),
        }
    }
}

impl std::fmt::Display for Comparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.operation.is_wildcard() {
            write!(f, "{}", self.operation)?;
        }

        write!(f, "{}", self.major)?;

        if let Some(minor) = self.minor {
            write!(f, ".{}", minor)?;

            if let Some(patch) = self.patch {
                write!(f, ".{}", patch)?;

                if let Some(pre) = &self.pre {
                    write!(f, "-{}", pre)?;
                }

                if let Some(build) = &self.build {
                    write!(f, "+{}", build)?;
                }
            } else if self.operation.is_wildcard() {
                write!(f, ".*")?;
            }
        } else if self.operation.is_wildcard() {
            write!(f, ".*")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum Op {
    Exact,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Tilde,
    #[default]
    Caret,
    Wildcard,
}

impl Op {
    fn parse_wildcard(input: &str) -> IResult<&str, char> {
        satisfy(|ch| ch == '*' || ch == 'x' || ch == 'X').parse(input)
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        macro_rules! op {
            ($parser:expr) => {
                delimited(space0, $parser, space0)
            };
        }

        alt((
            op!(tag(">=")).map(|_| Self::GreaterEq),
            op!(tag("<=")).map(|_| Self::LessEq),
            op!(tag("=")).map(|_| Self::Exact),
            op!(tag(">")).map(|_| Self::Greater),
            op!(tag("<")).map(|_| Self::Less),
            op!(tag("~")).map(|_| Self::Tilde),
            op!(tag("^")).map(|_| Self::Caret),
            op!(Self::parse_wildcard).map(|_| Self::Wildcard),
        ))
        .parse(input)
    }

    fn is_wildcard(&self) -> bool {
        matches!(self, Self::Wildcard)
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::GreaterEq => ">=",
                Op::LessEq => "<=",
                Op::Exact => "=",
                Op::Greater => ">",
                Op::Less => "<",
                Op::Tilde => "~",
                Op::Caret => "^",
                Op::Wildcard => "*",
            }
        )
    }
}

#[derive(Debug)]
enum VersionPart {
    Wildcard,
    Numeric(u64),
}

impl VersionPart {
    fn is_wildcard(&self) -> bool {
        matches!(self, Self::Wildcard)
    }

    fn is_numeric(&self) -> bool {
        matches!(self, Self::Numeric(_))
    }

    fn parse(input: &str) -> IResult<&str, Self> {
        preceded(
            char('.'),
            Op::parse_wildcard
                .map(|_| Self::Wildcard)
                .or(decimal(false).map(Self::Numeric)),
        )
        .parse(input)
    }

    fn resolve(self) -> Option<u64> {
        match self {
            VersionPart::Wildcard => None,
            VersionPart::Numeric(num) => Some(num),
        }
    }
}

// Taken and adapted from https://github.com/dtolnay/semver/blob/master/tests/test_version_req.rs
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::super::Version;
    use super::{Comparator, VersionReq};

    macro_rules! version {
        ($ver:expr) => {
            Version::from_str($ver).unwrap()
        };
    }

    macro_rules! req {
        ($req:expr) => {
            VersionReq::from_str($req).unwrap()
        };
    }

    macro_rules! comp {
        ($op:expr) => {
            Comparator::from_str($op).unwrap()
        };
    }

    #[track_caller]
    fn assert_match_all(req: &VersionReq, versions: &[&str]) {
        for string in versions {
            let parsed = version!(string);
            assert!(req.matches(&parsed), "{req} did not match {}", string);
        }
    }

    #[track_caller]
    fn assert_match_none(req: &VersionReq, versions: &[&str]) {
        for string in versions {
            let parsed = version!(string);
            assert!(!req.matches(&parsed), "{req} matched {}", string);
        }
    }

    #[track_caller]
    fn assert_to_string(req: impl std::fmt::Display, repr: &'static str) {
        assert_eq!(req.to_string(), repr);
    }

    #[test]
    fn basic() {
        let r = &req!("1.0.0");
        assert_to_string(r, "^1.0.0");
        assert_match_all(r, &["1.0.0", "1.1.0", "1.0.1"]);
        assert_match_none(r, &["0.9.9", "0.10.0", "0.1.0", "1.0.0-pre", "1.0.1-pre"]);
    }

    #[test]
    fn default() {
        let r = &VersionReq::default();
        assert_eq!(r, &VersionReq::any());
    }

    #[test]
    fn exact() {
        let r = &req!("=1.0.0");
        assert_to_string(r, "=1.0.0");
        assert_match_all(r, &["1.0.0"]);
        assert_match_none(r, &["1.0.1", "0.9.9", "0.10.0", "0.1.0", "1.0.0-pre"]);

        let r = &req!("=0.9.0");
        assert_to_string(r, "=0.9.0");
        assert_match_all(r, &["0.9.0"]);
        assert_match_none(r, &["0.9.1", "1.9.0", "0.0.9", "0.9.0-pre"]);

        let r = &req!("=0.0.2");
        assert_to_string(r, "=0.0.2");
        assert_match_all(r, &["0.0.2"]);
        assert_match_none(r, &["0.0.1", "0.0.3", "0.0.2-pre"]);

        let r = &req!("=0.1.0-beta2.a");
        assert_to_string(r, "=0.1.0-beta2.a");
        assert_match_all(r, &["0.1.0-beta2.a"]);
        assert_match_none(r, &["0.9.1", "0.1.0", "0.1.1-beta2.a", "0.1.0-beta2"]);

        let r = &req!("=0.1.0+meta");
        assert_to_string(r, "=0.1.0+meta");
        assert_match_all(r, &["0.1.0", "0.1.0+meta", "0.1.0+any"]);
    }

    #[test]
    pub fn greater_than() {
        let r = &req!(">= 1.0.0");
        assert_to_string(r, ">=1.0.0");
        assert_match_all(r, &["1.0.0", "2.0.0"]);
        assert_match_none(r, &["0.1.0", "0.0.1", "1.0.0-pre", "2.0.0-pre"]);

        let r = &req!(">= 2.1.0-alpha2");
        assert_to_string(r, ">=2.1.0-alpha2");
        assert_match_all(r, &["2.1.0-alpha2", "2.1.0-alpha3", "2.1.0", "3.0.0"]);
        assert_match_none(
            r,
            &["2.0.0", "2.1.0-alpha1", "2.0.0-alpha2", "3.0.0-alpha2"],
        );
    }

    #[test]
    pub fn less_than() {
        let r = &req!("< 1.0.0");
        assert_to_string(r, "<1.0.0");
        assert_match_all(r, &["0.1.0", "0.0.1"]);
        assert_match_none(r, &["1.0.0", "1.0.0-beta", "1.0.1", "0.9.9-alpha"]);

        let r = &req!("<= 2.1.0-alpha2");
        assert_match_all(r, &["2.1.0-alpha2", "2.1.0-alpha1", "2.0.0", "1.0.0"]);
        assert_match_none(
            r,
            &["2.1.0", "2.2.0-alpha1", "2.0.0-alpha2", "1.0.0-alpha2"],
        );

        let r = &req!(">1.0.0-alpha, <1.0.0");
        assert_match_all(r, &["1.0.0-beta"]);

        let r = &req!(">1.0.0-alpha, <1.0");
        assert_match_none(r, &["1.0.0-beta"]);

        let r = &req!(">1.0.0-alpha, <1");
        assert_match_none(r, &["1.0.0-beta"]);
    }

    #[test]
    pub fn multiple() {
        let r = &req!("> 0.0.9, <= 2.5.3");
        assert_to_string(r, ">0.0.9, <=2.5.3");
        assert_match_all(r, &["0.0.10", "1.0.0", "2.5.3"]);
        assert_match_none(r, &["0.0.8", "2.5.4"]);

        let r = &req!("0.3.0, 0.4.0");
        assert_to_string(r, "^0.3.0, ^0.4.0");
        assert_match_none(r, &["0.0.8", "0.3.0", "0.4.0"]);

        let r = &req!("<= 0.2.0, >= 0.5.0");
        assert_to_string(r, "<=0.2.0, >=0.5.0");
        assert_match_none(r, &["0.0.8", "0.3.0", "0.5.1"]);

        let r = &req!("0.1.0, 0.1.4, 0.1.6");
        assert_to_string(r, "^0.1.0, ^0.1.4, ^0.1.6");
        assert_match_all(r, &["0.1.6", "0.1.9"]);
        assert_match_none(r, &["0.1.0", "0.1.4", "0.2.0"]);

        assert!(VersionReq::from_str("> 0.1.0,").is_err()); // unexpected end of input while parsing major version number

        assert!(VersionReq::from_str("> 0.3.0, ,").is_err()); // unexpected character ',' while parsing major version number

        let r = &req!(">=0.5.1-alpha3, <0.6");
        assert_to_string(r, ">=0.5.1-alpha3, <0.6");
        assert_match_all(
            r,
            &[
                "0.5.1-alpha3",
                "0.5.1-alpha4",
                "0.5.1-beta",
                "0.5.1",
                "0.5.5",
            ],
        );
        assert_match_none(
            r,
            &["0.5.1-alpha1", "0.5.2-alpha3", "0.5.5-pre", "0.5.0-pre"],
        );
        assert_match_none(r, &["0.6.0", "0.6.0-pre"]);

        // https://github.com/steveklabnik/semver/issues/56
        assert!(VersionReq::from_str("1.2.3 - 2.3.4").is_err()); // expected comma after patch version number, found '-'
    }

    #[test]
    pub fn whitespace_delimited_operation_sets() {
        // https://github.com/steveklabnik/semver/issues/55
        assert!(VersionReq::from_str("> 0.0.9 <= 2.5.3").is_err()); // expected comma after patch version number, found '<'
    }

    #[test]
    pub fn tilde() {
        let r = &req!("~1");
        assert_match_all(r, &["1.0.0", "1.0.1", "1.1.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "0.0.9"]);

        let r = &req!("~1.2");
        assert_match_all(r, &["1.2.0", "1.2.1"]);
        assert_match_none(r, &["1.1.1", "1.3.0", "0.0.9"]);

        let r = &req!("~1.2.2");
        assert_match_all(r, &["1.2.2", "1.2.4"]);
        assert_match_none(r, &["1.2.1", "1.9.0", "1.0.9", "2.0.1", "0.1.3"]);

        let r = &req!("~1.2.3-beta.2");
        assert_match_all(r, &["1.2.3", "1.2.4", "1.2.3-beta.2", "1.2.3-beta.4"]);
        assert_match_none(r, &["1.3.3", "1.1.4", "1.2.3-beta.1", "1.2.4-beta.2"]);
    }

    #[test]
    pub fn caret() {
        let r = &req!("^1");
        assert_match_all(r, &["1.1.2", "1.1.0", "1.2.1", "1.0.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "0.1.4"]);
        assert_match_none(r, &["1.0.0-beta1", "0.1.0-alpha", "1.0.1-pre"]);

        let r = &req!("^1.1");
        assert_match_all(r, &["1.1.2", "1.1.0", "1.2.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.0.1", "0.1.4"]);

        let r = &req!("^1.1.2");
        assert_match_all(r, &["1.1.2", "1.1.4", "1.2.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
        assert_match_none(r, &["1.1.2-alpha1", "1.1.3-alpha1", "2.9.0-alpha1"]);

        let r = &req!("^0.1.2");
        assert_match_all(r, &["0.1.2", "0.1.4"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
        assert_match_none(r, &["0.1.2-beta", "0.1.3-alpha", "0.2.0-pre"]);

        let r = &req!("^0.5.1-alpha3");
        assert_match_all(
            r,
            &[
                "0.5.1-alpha3",
                "0.5.1-alpha4",
                "0.5.1-beta",
                "0.5.1",
                "0.5.5",
            ],
        );
        assert_match_none(
            r,
            &[
                "0.5.1-alpha1",
                "0.5.2-alpha3",
                "0.5.5-pre",
                "0.5.0-pre",
                "0.6.0",
            ],
        );

        let r = &req!("^0.0.2");
        assert_match_all(r, &["0.0.2"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1", "0.1.4"]);

        let r = &req!("^0.0");
        assert_match_all(r, &["0.0.2", "0.0.0"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.1.4"]);

        let r = &req!("^0");
        assert_match_all(r, &["0.9.1", "0.0.2", "0.0.0"]);
        assert_match_none(r, &["2.9.0", "1.1.1"]);

        let r = &req!("^1.4.2-beta.5");
        assert_match_all(
            r,
            &["1.4.2", "1.4.3", "1.4.2-beta.5", "1.4.2-beta.6", "1.4.2-c"],
        );
        assert_match_none(
            r,
            &[
                "0.9.9",
                "2.0.0",
                "1.4.2-alpha",
                "1.4.2-beta.4",
                "1.4.3-beta.5",
            ],
        );
    }

    #[test]
    pub fn wildcard() {
        assert!(VersionReq::from_str("").is_err()); // unexpected end of input while parsing major version number

        let r = &req!("*");
        assert_match_all(r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
        assert_match_none(r, &["1.0.0-pre"]);

        for s in &["x", "X"] {
            assert_eq!(*r, req!(s));
        }

        let r = &req!("1.*");
        assert_match_all(r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
        assert_match_none(r, &["0.0.9", "1.2.0-pre"]);

        for s in &["1.x", "1.X", "1.*.*"] {
            assert_eq!(*r, req!(s));
        }

        let r = &req!("1.2.*");
        assert_match_all(r, &["1.2.0", "1.2.2", "1.2.4"]);
        assert_match_none(r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3", "1.2.2-pre"]);

        for s in &["1.2.x", "1.2.X"] {
            assert_eq!(*r, req!(s));
        }
    }

    #[test]
    #[ignore = "logical ors are not implemented yet"]
    pub fn logical_or() {
        // https://github.com/steveklabnik/semver/issues/57
        assert!(VersionReq::from_str("=1.2.3 || =2.3.4").is_err()); // expected comma after patch version number, found '|'
        assert!(VersionReq::from_str("1.1 || =1.2.3").is_err()); // expected comma after minor version number, found '|'
        assert!(VersionReq::from_str("6.* || 8.* || >= 10.*").is_err()); // expected comma after minor version number, found '|'
    }

    #[test]
    pub fn any() {
        let r = &VersionReq::any();
        assert_match_all(r, &["0.0.1", "0.1.0", "1.0.0"]);
    }

    #[test]
    pub fn pre() {
        let r = &req!("=2.1.1-really.0");
        assert_match_all(r, &["2.1.1-really.0"]);
    }

    #[test]
    pub fn parse() {
        assert!(VersionReq::from_str("\0").is_err()); // unexpected character '\\0' while parsing major version number
        assert!(VersionReq::from_str(">= >= 0.0.2").is_err()); // unexpected character '>' while parsing major version number
        assert!(VersionReq::from_str(">== 0.0.2").is_err()); // unexpected character '=' while parsing major version number
        assert!(VersionReq::from_str("a.0.0").is_err()); // unexpected character 'a' while parsing major version number
        assert!(VersionReq::from_str("1.0.0-").is_err()); // empty identifier segment in pre-release identifier
        assert!(VersionReq::from_str(">=").is_err()); // unexpected end of input while parsing major version number
    }

    #[test]
    fn operation_parse() {
        let parsed = comp!("1.2.3-alpha");
        assert_to_string(parsed, "^1.2.3-alpha");

        let parsed = comp!("2.X");
        assert_to_string(parsed, "2.*");

        let parsed = comp!("2");
        assert_to_string(parsed, "^2");

        let parsed = comp!("2.x.x");
        assert_to_string(parsed, "2.*");

        assert!(Comparator::from_str("1.2.3-01").is_err()); // invalid leading zero in pre-release identifier
        assert!(Comparator::from_str("1.2.3+4.").is_err()); // empty identifier segment in build metadata
        assert!(Comparator::from_str(">").is_err()); // unexpected end of input while parsing major version number
        assert!(Comparator::from_str("1.").is_err()); // unexpected end of input while parsing minor version number
        assert!(Comparator::from_str("1.*.").is_err()); // unexpected character after wildcard in version req
        assert!(Comparator::from_str("1.2.3+4ÿ").is_err()); // unexpected character 'ÿ' after build metadata
    }

    #[test]
    fn cargo3202() {
        let r = &req!("0.*.*");
        assert_to_string(r, "0.*");
        assert_match_all(r, &["0.5.0"]);

        let r = &req!("0.0.*");
        assert_to_string(r, "0.0.*");
    }

    #[test]
    fn digit_after_wildcard() {
        assert!(VersionReq::from_str("*.1").is_err()); // unexpected character after wildcard in version req
        assert!(VersionReq::from_str("1.*.1").is_err()); // unexpected character after wildcard in version req
        assert!(VersionReq::from_str(">=1.*.1").is_err()); // unexpected character after wildcard in version req
    }

    #[test]
    fn leading_digit_in_pre_and_build() {
        for op in &["=", ">", ">=", "<", "<=", "~", "^"] {
            // digit then alpha
            req!(&format!("{} 1.2.3-1a", op));
            req!(&format!("{} 1.2.3+1a", op));

            // digit then alpha (leading zero)
            req!(&format!("{} 1.2.3-01a", op));
            req!(&format!("{} 1.2.3+01", op));

            // multiple
            req!(&format!("{} 1.2.3-1+1", op));
            req!(&format!("{} 1.2.3-1-1+1-1-1", op));
            req!(&format!("{} 1.2.3-1a+1a", op));
            req!(&format!("{} 1.2.3-1a-1a+1a-1a-1a", op));
        }
    }

    #[test]
    fn wildcard_and_another() {
        assert!(VersionReq::from_str("*, 0.20.0-any").is_err()); // wildcard req (*) must be the only operation in the version req
        assert!(VersionReq::from_str("0.20.0-any, *").is_err()); // wildcard req (*) must be the only operation in the version req
        assert!(VersionReq::from_str("0.20.0-any, *, 1.0").is_err()); // wildcard req (*) must be the only operation in the version req
    }
}
