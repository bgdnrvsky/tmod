use super::utils::*;

use std::fmt::Display;

use itertools::Itertools;
use nom::{
    character::complete::{char, digit1, one_of},
    combinator::{all_consuming, cond, map_res},
    multi::{many1, separated_list1},
    sequence::preceded,
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};

/// Custom implementation of semver Version.
/// Needed since the `semver` crate isn't flexible enough
#[derive(Debug, Clone, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
    pre: Option<PreRelease>,
    build: Option<BuildMetadata>,
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
            .then(self.pre.cmp(&other.pre))
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
    }
}

impl Eq for Version {}

impl Version {
    fn parse(input: &str) -> IResult<&str, Self> {
        version_core
            .and(opt(PreRelease::parse))
            .and(opt(BuildMetadata::parse))
            .map(|(((major, minor, patch), pre), build)| Self {
                major,
                minor,
                patch,
                pre,
                build,
            })
            .parse(input)
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
            .map(String::from_iter)
            .map(Self::Textual);

        numeric.or(textual).parse(s)
    }
}

impl std::str::FromStr for Identifier {
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
        preceded(char('-'), separated_list1(char('.'), Identifier::parse))
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
        preceded(char('+'), separated_list1(char('.'), Identifier::parse))
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

    use super::{BuildMetadata, PreRelease as Prerelease, Version};

    fn version(s: &str) -> Version {
        Version::from_str(s).unwrap()
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
            version("1.2.3"),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: None,
                build: None
            }
        );

        assert_eq!(
            version("1.2.3-alpha1"),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: Some(Prerelease::from_str("alpha1").unwrap()),
                build: None
            }
        );

        let parsed = version("1.2.3+build5");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: Some(BuildMetadata::from_str("build5").unwrap()),
        };
        assert_eq!(parsed, expected);

        let parsed = version("1.2.3+5build");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: None,
            build: Some(BuildMetadata::from_str("5build").unwrap()),
        };
        assert_eq!(parsed, expected);

        let parsed = version("1.2.3-alpha1+build5");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(Prerelease::from_str("alpha1").unwrap()),
            build: Some(BuildMetadata::from_str("build5").unwrap()),
        };
        assert_eq!(parsed, expected);

        let parsed = version("1.2.3-1.alpha1.9+build5.7.3aedf");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(Prerelease::from_str("1.alpha1.9").unwrap()),
            build: Some(BuildMetadata::from_str("build5.7.3aedf").unwrap()),
        };
        assert_eq!(parsed, expected);

        let parsed = version("1.2.3-0a.alpha1.9+05build.7.3aedf");
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some(Prerelease::from_str("0a.alpha1.9").unwrap()),
            build: Some(BuildMetadata::from_str("05build.7.3aedf").unwrap()),
        };
        assert_eq!(parsed, expected);

        let parsed = version("0.4.0-beta.1+0851523");
        let expected = Version {
            major: 0,
            minor: 4,
            patch: 0,
            pre: Some(Prerelease::from_str("beta.1").unwrap()),
            build: Some(BuildMetadata::from_str("0851523").unwrap()),
        };
        assert_eq!(parsed, expected);

        // for https://nodejs.org/dist/index.json, where some older npm versions are "1.1.0-beta-10"
        let parsed = version("1.1.0-beta-10");
        let expected = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: Some(Prerelease::from_str("beta-10").unwrap()),
            build: None,
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn eq() {
        assert_eq!(version("1.2.3"), version("1.2.3"));
        assert_eq!(version("1.2.3-alpha1"), version("1.2.3-alpha1"));
        assert_eq!(version("1.2.3+build.42"), version("1.2.3+build.42"));
        assert_eq!(version("1.2.3-alpha1+42"), version("1.2.3-alpha1+42"));
    }

    #[test]
    fn ne() {
        assert_ne!(version("0.0.0"), version("0.0.1"));
        assert_ne!(version("0.0.0"), version("0.1.0"));
        assert_ne!(version("0.0.0"), version("1.0.0"));
        assert_ne!(version("1.2.3-alpha"), version("1.2.3-beta"));
        assert_ne!(version("1.2.3+23"), version("1.2.3+42"));
    }

    #[test]
    fn display() {
        assert_eq!(version("1.2.3").to_string(), "1.2.3");
        assert_eq!(version("1.2.3-alpha1").to_string(), "1.2.3-alpha1");
        assert_eq!(version("1.2.3+build.42").to_string(), "1.2.3+build.42");
        assert_eq!(version("1.2.3-alpha1+42").to_string(), "1.2.3-alpha1+42");
    }

    #[test]
    fn lt() {
        assert!(version("0.0.0") < version("1.2.3-alpha2"));
        assert!(version("1.0.0") < version("1.2.3-alpha2"));
        assert!(version("1.2.0") < version("1.2.3-alpha2"));
        assert!(version("1.2.3-alpha1") < version("1.2.3"));
        assert!(version("1.2.3-alpha1") < version("1.2.3-alpha2"));
        assert!(version("1.2.3-alpha2") >= version("1.2.3-alpha2"));
        assert!(version("1.2.3+23") < version("1.2.3+42"));
    }

    #[test]
    fn le() {
        assert!(version("0.0.0") <= version("1.2.3-alpha2"));
        assert!(version("1.0.0") <= version("1.2.3-alpha2"));
        assert!(version("1.2.0") <= version("1.2.3-alpha2"));
        assert!(version("1.2.3-alpha1") <= version("1.2.3-alpha2"));
        assert!(version("1.2.3-alpha2") <= version("1.2.3-alpha2"));
        assert!(version("1.2.3+23") <= version("1.2.3+42"));
    }

    #[test]
    fn gt() {
        assert!(version("1.2.3-alpha2") > version("0.0.0"));
        assert!(version("1.2.3-alpha2") > version("1.0.0"));
        assert!(version("1.2.3-alpha2") > version("1.2.0"));
        assert!(version("1.2.3-alpha2") > version("1.2.3-alpha1"));
        assert!(version("1.2.3") > version("1.2.3-alpha2"));
        assert!(version("1.2.3-alpha2") <= version("1.2.3-alpha2"));
        assert!(version("1.2.3+23") <= version("1.2.3+42"));
    }

    #[test]
    fn ge() {
        assert!(version("1.2.3-alpha2") >= version("0.0.0"));
        assert!(version("1.2.3-alpha2") >= version("1.0.0"));
        assert!(version("1.2.3-alpha2") >= version("1.2.0"));
        assert!(version("1.2.3-alpha2") >= version("1.2.3-alpha1"));
        assert!(version("1.2.3-alpha2") >= version("1.2.3-alpha2"));
        assert!(version("1.2.3+23") < version("1.2.3+42"));
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
            let a = version(vs[i - 1]);
            let b = version(vs[i]);
            assert!(a < b, "nope {:?} < {:?}", a, b);
            i += 1;
        }
    }
}
