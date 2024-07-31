use super::utils::*;

use std::fmt::Display;

use itertools::Itertools;
use nom::{
    bytes::complete::take_while1,
    character::complete::char,
    combinator::{all_consuming, cond, map_res},
    multi::separated_list1,
    sequence::preceded,
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// Custom implementation of semver Version.
/// Needed since the `semver` crate isn't flexible enough
#[derive(Debug, Clone, DeserializeFromStr, SerializeDisplay)]
pub struct Version {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
    pub(crate) pre: Option<PreRelease>,
    pub(crate) build: Option<BuildMetadata>,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
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
            && self.build.eq(&other.build)
    }
}

impl Eq for Version {}

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.major.hash(state);
        self.minor.hash(state);
        self.patch.hash(state);
        self.pre.hash(state);
        self.build.hash(state);
    }
}

impl Version {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (rest, (major, minor, patch)) = version_core(input)?;
        let (rest, pre) = cond(rest.starts_with('-'), PreRelease::parse).parse(rest)?;
        let (rest, build) = cond(rest.starts_with('+'), BuildMetadata::parse).parse(rest)?;

        Ok((
            rest,
            Self {
                major,
                minor,
                patch,
                pre,
                build,
            },
        ))
    }
}

impl std::str::FromStr for Version {
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

/// Identifiers MUST comprise only ASCII alphanumerics and hyphens [0-9A-Za-z-].
/// Identifiers MUST NOT be empty.
/// Numeric identifiers MUST NOT include leading zeroes. (except build metadata)
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum Identifier {
    Numeric(u64),
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

impl Ord for Identifier {
    // 1. Identifiers with letters or hyphens are compared lexically in ASCII sort order.
    // 2. Numeric identifiers always have lower precedence than non-numeric identifiers.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Identifier::Numeric(a), Identifier::Numeric(b)) => a.cmp(b),
            (Identifier::Numeric(_), Identifier::Textual(_)) => std::cmp::Ordering::Less,
            (Identifier::Textual(_), Identifier::Numeric(_)) => std::cmp::Ordering::Greater,
            (Identifier::Textual(a), Identifier::Textual(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Identifier {
    fn parse(accept_zeros: bool) -> impl FnMut(&str) -> IResult<&str, Self> {
        move |input| {
            map_res(
                take_while1(|ch: char| ch == '-' || ch.is_ascii_alphanumeric()),
                |out: &str| {
                    if out.contains(|ch: char| ch == '-' || ch.is_ascii_alphabetic()) {
                        Ok(Self::Textual(out.to_string()))
                    } else {
                        decimal(accept_zeros)
                            .map(Self::Numeric)
                            .parse(out)
                            .map(|(_, numeric)| numeric)
                    }
                },
            )
            .parse(input)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct PreRelease {
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
    pub(crate) fn parse(input: &str) -> IResult<&str, Self> {
        preceded(
            char('-'),
            separated_list1(char('.'), Identifier::parse(false)),
        )
        .map(|idents| Self { idents })
        .parse(input)
    }
}

impl std::str::FromStr for PreRelease {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct BuildMetadata {
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
    pub fn parse(input: &str) -> IResult<&str, Self> {
        preceded(
            char('+'),
            separated_list1(char('.'), Identifier::parse(true)),
        )
        .map(|idents| Self { idents })
        .parse(input)
    }
}

impl std::str::FromStr for BuildMetadata {
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

// Taken and adapted from https://github.com/dtolnay/semver/blob/master/tests/test_version.rs
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{BuildMetadata, Identifier, PreRelease as Prerelease, Version};

    macro_rules! version {
        ($ver:expr) => {
            Version::from_str($ver).unwrap()
        };
    }

    macro_rules! ident {
        (text $txt:literal) => {
            Identifier::Textual(String::from($txt))
        };
        (num $num:literal) => {
            Identifier::Numeric($num)
        };
    }

    macro_rules! pre {
        [$( $ident:expr ),*] => {
            Some(Prerelease { idents: vec![ $( $ident ),* ] })
        };
    }

    macro_rules! build {
        [$( $ident:expr ),*] => {
            Some(BuildMetadata { idents: vec![ $( $ident ),* ] })
        };
    }

    #[test]
    fn parse() {
        assert!(Version::from_str("").is_err()); // empty string, expected a semver version
        assert!(Version::from_str("  ").is_err()); // unexpected character ' ' while parsing major version number
        assert!(Version::from_str("1").is_err()); // unexpected end of input while parsing major version number
        assert!(Version::from_str("1.2").is_err()); // unexpected end of input while parsing minor version number
        assert!(Version::from_str("1.2.3-").is_err()); // empty identifier segment in pre-release identifier
        assert!(Version::from_str("a.b.c").is_err()); // unexpected character 'a' while parsing major version number
        assert!(Version::from_str("1.2.3 abc").is_err()); // unexpected character ' ' after patch version number
        assert!(Version::from_str("1.2.3-01").is_err()); // invalid leading zero in pre-release identifier
        assert!(Version::from_str("1.2.3++").is_err()); // empty identifier segment in build metadata
        assert!(Version::from_str("07").is_err()); // invalid leading zero in major version number
        assert!(Version::from_str("111111111111111111111.0.0").is_err()); // value of major version number exceeds u64::MAX
        assert!(Version::from_str("8\0").is_err()); // unexpected character '\\0' after major version number

        assert_eq!(
            version!("1.2.3"),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: None,
                build: None
            }
        );

        assert_eq!(
            version!("1.2.3-alpha1"),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: pre![ident!(text "alpha1")],
                build: None
            }
        );

        let parsed = version!("1.2.3+build5");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: build![ident!(text "build5")],
        };
        assert_eq!(parsed, expected);

        let parsed = version!("1.2.3+5build");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: build![ident!(text "5build")],
        };
        assert_eq!(parsed, expected);

        let parsed = version!("1.2.3-alpha1+build5");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: pre![ident!(text "alpha1")],
            build: build![ident!(text "build5")],
        };
        assert_eq!(parsed, expected);

        let parsed = version!("1.2.3-1.alpha1.9+build5.7.3aedf");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: pre![ident!(num 1), ident!(text "alpha1"), ident!(num 9)],
            build: build![ident!(text "build5"), ident!(num 7), ident!(text "3aedf")],
        };
        assert_eq!(parsed, expected);

        let parsed = version!("1.2.3-0a.alpha1.9+05build.7.3aedf");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: pre![ident!(text "0a"), ident!(text "alpha1"), ident!(num 9)],
            build: build![ident!(text "05build"), ident!(num 7), ident!(text "3aedf")],
        };
        assert_eq!(parsed, expected);

        let parsed = version!("0.4.0-beta.1+0851523");
        let expected = Version {
            major: 0,
            minor: 4,
            patch: 0,
            pre: pre![ident!(text "beta"), ident!(num 1)],
            build: build![ident!(num 851523)],
        };
        assert_eq!(parsed, expected);

        // for https://nodejs.org/dist/index.json, where some older npm versions are "1.1.0-beta-10"
        let parsed = version!("1.1.0-beta-10");
        let expected = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: pre![ident!(text "beta-10")],
            build: None,
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn eq() {
        assert_eq!(version!("1.2.3"), version!("1.2.3"));
        assert_eq!(version!("1.2.3-alpha1"), version!("1.2.3-alpha1"));
        assert_eq!(version!("1.2.3+build.42"), version!("1.2.3+build.42"));
        assert_eq!(version!("1.2.3-alpha1+42"), version!("1.2.3-alpha1+42"));
    }

    #[test]
    fn ne() {
        assert_ne!(version!("0.0.0"), version!("0.0.1"));
        assert_ne!(version!("0.0.0"), version!("0.1.0"));
        assert_ne!(version!("0.0.0"), version!("1.0.0"));
        assert_ne!(version!("1.2.3-alpha"), version!("1.2.3-beta"));
    }

    #[test]
    fn display() {
        assert_eq!(version!("1.2.3").to_string(), "1.2.3");
        assert_eq!(version!("1.2.3-alpha1").to_string(), "1.2.3-alpha1");
        assert_eq!(version!("1.2.3+build.42").to_string(), "1.2.3+build.42");
        assert_eq!(version!("1.2.3-alpha1+42").to_string(), "1.2.3-alpha1+42");
    }

    #[test]
    fn lt() {
        assert!(version!("0.0.0") < version!("1.2.3-alpha2"));
        assert!(version!("1.0.0") < version!("1.2.3-alpha2"));
        assert!(version!("1.2.0") < version!("1.2.3-alpha2"));
        assert!(version!("1.2.3-alpha1") < version!("1.2.3"));
        assert!(version!("1.2.3-alpha1") < version!("1.2.3-alpha2"));
        assert!(version!("1.2.3-alpha2") >= version!("1.2.3-alpha2"));
    }

    #[test]
    fn le() {
        assert!(version!("0.0.0") <= version!("1.2.3-alpha2"));
        assert!(version!("1.0.0") <= version!("1.2.3-alpha2"));
        assert!(version!("1.2.0") <= version!("1.2.3-alpha2"));
        assert!(version!("1.2.3-alpha1") <= version!("1.2.3-alpha2"));
        assert!(version!("1.2.3-alpha2") <= version!("1.2.3-alpha2"));
    }

    #[test]
    fn gt() {
        assert!(version!("1.2.3-alpha2") > version!("0.0.0"));
        assert!(version!("1.2.3-alpha2") > version!("1.0.0"));
        assert!(version!("1.2.3-alpha2") > version!("1.2.0"));
        assert!(version!("1.2.3-alpha2") > version!("1.2.3-alpha1"));
        assert!(version!("1.2.3") > version!("1.2.3-alpha2"));
        assert!(version!("1.2.3-alpha2") <= version!("1.2.3-alpha2"));
    }

    #[test]
    fn ge() {
        assert!(version!("1.2.3-alpha2") >= version!("0.0.0"));
        assert!(version!("1.2.3-alpha2") >= version!("1.0.0"));
        assert!(version!("1.2.3-alpha2") >= version!("1.2.0"));
        assert!(version!("1.2.3-alpha2") >= version!("1.2.3-alpha1"));
        assert!(version!("1.2.3-alpha2") >= version!("1.2.3-alpha2"));
    }

    #[test]
    fn spec_order() {
        let vs = [
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
        ];
        let mut i = 1;
        while i < vs.len() {
            let a = version!(vs[i - 1]);
            let b = version!(vs[i]);
            assert!(a < b, "nope {:?} < {:?}", a, b);
            i += 1;
        }
    }
}
