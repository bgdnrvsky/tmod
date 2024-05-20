use nom::{
    character::complete::{alpha1, digit1, one_of},
    combinator::map_res,
    multi::separated_list1,
    IResult, Parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionItem {
    Numeric(usize),
    Textual(String),
}

impl VersionItem {
    fn parse(s: &str) -> IResult<&str, Self> {
        map_res(digit1, str::parse::<usize>)
            .map(Self::Numeric)
            .or(alpha1
                .map(|value: &str| value.to_lowercase())
                .map(Self::Textual))
            .parse(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    items: Vec<VersionItem>,
}

impl Version {
    fn parse(s: &str) -> IResult<&str, Self> {
        separated_list1(one_of(".-"), VersionItem::parse)
            .map(|items| Self { items })
            .parse(s)
    }

    fn new(items: Vec<VersionItem>) -> Self {
        Self { items }
    }
}

#[cfg(test)]
mod versions {
    use super::*;

    #[test]
    fn version() {
        assert_eq!(
            Version::parse("1.2.3.4.5"),
            Ok((
                "",
                Version::new((1..=5).map(VersionItem::Numeric).collect())
            ))
        );

        assert_eq!(
            Version::parse("1.20.1"),
            Ok((
                "",
                Version::new([1, 20, 1].into_iter().map(VersionItem::Numeric).collect())
            ))
        );

        assert_eq!(
            Version::parse("1.20-SNAPSHOT"),
            Ok((
                "",
                Version::new(vec![
                    VersionItem::Numeric(1),
                    VersionItem::Numeric(20),
                    VersionItem::Textual(String::from("snapshot"))
                ])
            ))
        );

        assert!(Version::parse(".10").is_err());
    }

    #[test]
    fn version_item_numeric() {
        assert_eq!(
            VersionItem::parse("123"),
            Ok(("", VersionItem::Numeric(123)))
        );

        assert_eq!(VersionItem::parse("003"), Ok(("", VersionItem::Numeric(3))));
    }

    #[test]
    fn version_item_textual() {
        assert_eq!(
            VersionItem::parse("SNAPSHOT"),
            Ok(("", VersionItem::Textual(String::from("snapshot"))))
        );
    }
}
