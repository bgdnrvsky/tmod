use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, satisfy, space0},
    combinator::opt,
    multi::separated_list1,
    sequence::{preceded, terminated},
    Finish, IResult, Parser,
};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum_macros::Display;

use super::utils::decimal;
use super::version::PreRelease;
use super::version::{BuildMetadata, PreRelease};

#[derive(Display, Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
enum VersionPattern {
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
    WithPre {
        major: usize,
        minor: usize,
        patch: usize,
        pre: PreRelease,
    },
    #[strum(to_string = "{major}.{minor}.{patch}+{build}")]
    WithBuild {
        major: usize,
        minor: usize,
        patch: usize,
        build: BuildMetadata,
    },
    #[strum(to_string = "{major}.{minor}.{patch}-{pre}+{build}")]
    WitPreAndBuild {
        major: usize,
        minor: usize,
        patch: usize,
        pre: PreRelease,
        build: BuildMetadata,
    },
}

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
impl VersionPattern {
    }
}

impl std::str::FromStr for VersionPattern {
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
    Exact(VersionPattern),
    #[strum(to_string = ">{0}")]
    Greater(VersionPattern),
    #[strum(to_string = ">={0}")]
    GreaterEq(VersionPattern),
    #[strum(to_string = "<{0}")]
    Less(VersionPattern),
    #[strum(to_string = "<={0}")]
    LessEq(VersionPattern),
    #[strum(to_string = "~{0}")]
    Tilde(VersionPattern),
    #[strum(to_string = "^{0}")]
    Caret(VersionPattern),
    #[strum(to_string = "*")]
    Wildcard,
}

impl Op {
    fn parse(input: &str) -> IResult<&str, Self> {
        let op = |prefix| preceded(tag(prefix), VersionPattern::parse);
        alt((
            op(">=").map(Self::GreaterEq),
            op("<=").map(Self::LessEq),
            op("=").map(Self::Exact),
            op(">").map(Self::Greater),
            op("<").map(Self::Less),
            op("~").map(Self::Tilde),
            op("^").map(Self::Caret),
            satisfy(|ch: char| ch == '*' || ch == 'x' || ch == 'X').map(|_| Self::Wildcard),
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

impl Default for VersionReq {
    fn default() -> Self {
        Self::any()
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

// Taken and adapted from https://github.com/dtolnay/semver/blob/master/tests/test_version_req.rs
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::super::Version;
    use super::{Op, VersionReq};

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

    macro_rules! op {
        ($op:expr) => {
            Op::from_str($op).unwrap()
        };
    }

    fn assert_match_all(req: &VersionReq, versions: &[&str]) {
        for string in versions {
            let parsed = version!(string);
            assert!(req.matches(&parsed), "{req} did not match {}", string);
        }
    }

    fn assert_match_none(req: &VersionReq, versions: &[&str]) {
        for string in versions {
            let parsed = version!(string);
            assert!(!req.matches(&parsed), "{req} matched {}", string);
        }
    }

    fn assert_to_string(req: impl std::fmt::Display, repr: &'static str) {
        assert_eq!(req.to_string(), repr);
    }

    #[test]
    fn basic() {
        let ref r = req!("1.0.0");
        assert_to_string(r, "^1.0.0");
        assert_match_all(r, &["1.0.0", "1.1.0", "1.0.1"]);
        assert_match_none(r, &["0.9.9", "0.10.0", "0.1.0", "1.0.0-pre", "1.0.1-pre"]);
    }

    #[test]
    fn default() {
        let ref r = VersionReq::default();
        assert_eq!(r, &VersionReq::any());
    }

    #[test]
    fn exact() {
        let ref r = req!("=1.0.0");
        assert_to_string(r, "=1.0.0");
        assert_match_all(r, &["1.0.0"]);
        assert_match_none(r, &["1.0.1", "0.9.9", "0.10.0", "0.1.0", "1.0.0-pre"]);

        let ref r = req!("=0.9.0");
        assert_to_string(r, "=0.9.0");
        assert_match_all(r, &["0.9.0"]);
        assert_match_none(r, &["0.9.1", "1.9.0", "0.0.9", "0.9.0-pre"]);

        let ref r = req!("=0.0.2");
        assert_to_string(r, "=0.0.2");
        assert_match_all(r, &["0.0.2"]);
        assert_match_none(r, &["0.0.1", "0.0.3", "0.0.2-pre"]);

        let ref r = req!("=0.1.0-beta2.a");
        assert_to_string(r, "=0.1.0-beta2.a");
        assert_match_all(r, &["0.1.0-beta2.a"]);
        assert_match_none(r, &["0.9.1", "0.1.0", "0.1.1-beta2.a", "0.1.0-beta2"]);

        let ref r = req!("=0.1.0+meta");
        assert_to_string(r, "=0.1.0");
        assert_match_all(r, &["0.1.0", "0.1.0+meta", "0.1.0+any"]);
    }

    #[test]
    pub fn greater_than() {
        let ref r = req!(">= 1.0.0");
        assert_to_string(r, ">=1.0.0");
        assert_match_all(r, &["1.0.0", "2.0.0"]);
        assert_match_none(r, &["0.1.0", "0.0.1", "1.0.0-pre", "2.0.0-pre"]);

        let ref r = req!(">= 2.1.0-alpha2");
        assert_to_string(r, ">=2.1.0-alpha2");
        assert_match_all(r, &["2.1.0-alpha2", "2.1.0-alpha3", "2.1.0", "3.0.0"]);
        assert_match_none(
            r,
            &["2.0.0", "2.1.0-alpha1", "2.0.0-alpha2", "3.0.0-alpha2"],
        );
    }

    #[test]
    pub fn less_than() {
        let ref r = req!("< 1.0.0");
        assert_to_string(r, "<1.0.0");
        assert_match_all(r, &["0.1.0", "0.0.1"]);
        assert_match_none(r, &["1.0.0", "1.0.0-beta", "1.0.1", "0.9.9-alpha"]);

        let ref r = req!("<= 2.1.0-alpha2");
        assert_match_all(r, &["2.1.0-alpha2", "2.1.0-alpha1", "2.0.0", "1.0.0"]);
        assert_match_none(
            r,
            &["2.1.0", "2.2.0-alpha1", "2.0.0-alpha2", "1.0.0-alpha2"],
        );

        let ref r = req!(">1.0.0-alpha, <1.0.0");
        assert_match_all(r, &["1.0.0-beta"]);

        let ref r = req!(">1.0.0-alpha, <1.0");
        assert_match_none(r, &["1.0.0-beta"]);

        let ref r = req!(">1.0.0-alpha, <1");
        assert_match_none(r, &["1.0.0-beta"]);
    }

    #[test]
    pub fn multiple() {
        let ref r = req!("> 0.0.9, <= 2.5.3");
        assert_to_string(r, ">0.0.9, <=2.5.3");
        assert_match_all(r, &["0.0.10", "1.0.0", "2.5.3"]);
        assert_match_none(r, &["0.0.8", "2.5.4"]);

        let ref r = req!("0.3.0, 0.4.0");
        assert_to_string(r, "^0.3.0, ^0.4.0");
        assert_match_none(r, &["0.0.8", "0.3.0", "0.4.0"]);

        let ref r = req!("<= 0.2.0, >= 0.5.0");
        assert_to_string(r, "<=0.2.0, >=0.5.0");
        assert_match_none(r, &["0.0.8", "0.3.0", "0.5.1"]);

        let ref r = req!("0.1.0, 0.1.4, 0.1.6");
        assert_to_string(r, "^0.1.0, ^0.1.4, ^0.1.6");
        assert_match_all(r, &["0.1.6", "0.1.9"]);
        assert_match_none(r, &["0.1.0", "0.1.4", "0.2.0"]);

        assert!(VersionReq::from_str("> 0.1.0,").is_err()); // unexpected end of input while parsing major version number

        assert!(VersionReq::from_str("> 0.3.0, ,").is_err()); // unexpected character ',' while parsing major version number

        let ref r = req!(">=0.5.1-alpha3, <0.6");
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

        #[rustfmt::skip]
        // excessive number of version operations
        assert!(VersionReq::from_str(">1, >2, >3, >4, >5,\
                                      >6, >7, >8, >9, >10,\
                                      >11, >12, >13, >14,\
                                      >15, >16, >17, >18,\
                                      >19, >20, >21, >22,\
                                      >23, >24, >25, >26,\
                                      >27, >28, >29, >30,\
                                      >31, >32, >33").is_err());
    }

    #[test]
    pub fn whitespace_delimited_operation_sets() {
        // https://github.com/steveklabnik/semver/issues/55
        assert!(VersionReq::from_str("> 0.0.9 <= 2.5.3").is_err()); // expected comma after patch version number, found '<'
    }

    #[test]
    pub fn tilde() {
        let ref r = req!("~1");
        assert_match_all(r, &["1.0.0", "1.0.1", "1.1.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "0.0.9"]);

        let ref r = req!("~1.2");
        assert_match_all(r, &["1.2.0", "1.2.1"]);
        assert_match_none(r, &["1.1.1", "1.3.0", "0.0.9"]);

        let ref r = req!("~1.2.2");
        assert_match_all(r, &["1.2.2", "1.2.4"]);
        assert_match_none(r, &["1.2.1", "1.9.0", "1.0.9", "2.0.1", "0.1.3"]);

        let ref r = req!("~1.2.3-beta.2");
        assert_match_all(r, &["1.2.3", "1.2.4", "1.2.3-beta.2", "1.2.3-beta.4"]);
        assert_match_none(r, &["1.3.3", "1.1.4", "1.2.3-beta.1", "1.2.4-beta.2"]);
    }

    #[test]
    pub fn caret() {
        let ref r = req!("^1");
        assert_match_all(r, &["1.1.2", "1.1.0", "1.2.1", "1.0.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "0.1.4"]);
        assert_match_none(r, &["1.0.0-beta1", "0.1.0-alpha", "1.0.1-pre"]);

        let ref r = req!("^1.1");
        assert_match_all(r, &["1.1.2", "1.1.0", "1.2.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.0.1", "0.1.4"]);

        let ref r = req!("^1.1.2");
        assert_match_all(r, &["1.1.2", "1.1.4", "1.2.1"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
        assert_match_none(r, &["1.1.2-alpha1", "1.1.3-alpha1", "2.9.0-alpha1"]);

        let ref r = req!("^0.1.2");
        assert_match_all(r, &["0.1.2", "0.1.4"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
        assert_match_none(r, &["0.1.2-beta", "0.1.3-alpha", "0.2.0-pre"]);

        let ref r = req!("^0.5.1-alpha3");
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

        let ref r = req!("^0.0.2");
        assert_match_all(r, &["0.0.2"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1", "0.1.4"]);

        let ref r = req!("^0.0");
        assert_match_all(r, &["0.0.2", "0.0.0"]);
        assert_match_none(r, &["0.9.1", "2.9.0", "1.1.1", "0.1.4"]);

        let ref r = req!("^0");
        assert_match_all(r, &["0.9.1", "0.0.2", "0.0.0"]);
        assert_match_none(r, &["2.9.0", "1.1.1"]);

        let ref r = req!("^1.4.2-beta.5");
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

        let ref r = req!("*");
        assert_match_all(r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
        assert_match_none(r, &["1.0.0-pre"]);

        for s in &["x", "X"] {
            assert_eq!(*r, req!(s));
        }

        let ref r = req!("1.*");
        assert_match_all(r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
        assert_match_none(r, &["0.0.9", "1.2.0-pre"]);

        for s in &["1.x", "1.X", "1.*.*"] {
            assert_eq!(*r, req!(s));
        }

        let ref r = req!("1.2.*");
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
        let ref r = VersionReq::any();
        assert_match_all(r, &["0.0.1", "0.1.0", "1.0.0"]);
    }

    #[test]
    pub fn pre() {
        let ref r = req!("=2.1.1-really.0");
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
        let parsed = op!("1.2.3-alpha");
        assert_to_string(parsed, "^1.2.3-alpha");

        let parsed = op!("2.X");
        assert_to_string(parsed, "2.*");

        let parsed = op!("2");
        assert_to_string(parsed, "^2");

        let parsed = op!("2.x.x");
        assert_to_string(parsed, "2.*");

        assert!(Op::from_str("1.2.3-01").is_err()); // invalid leading zero in pre-release identifier
        assert!(Op::from_str("1.2.3+4.").is_err()); // empty identifier segment in build metadata
        assert!(Op::from_str(">").is_err()); // unexpected end of input while parsing major version number
        assert!(Op::from_str("1.").is_err()); // unexpected end of input while parsing minor version number
        assert!(Op::from_str("1.*.").is_err()); // unexpected character after wildcard in version req
        assert!(Op::from_str("1.2.3+4ÿ").is_err()); // unexpected character 'ÿ' after build metadata
    }

    #[test]
    fn cargo3202() {
        let ref r = req!("0.*.*");
        assert_to_string(r, "0.*");
        assert_match_all(r, &["0.5.0"]);

        let ref r = req!("0.0.*");
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
