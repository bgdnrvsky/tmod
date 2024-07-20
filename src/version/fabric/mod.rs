mod req;
mod version;

pub use req::VersionReq;
pub use version::Version;

pub(crate) mod utils {
    use nom::{
        character::complete::{char, digit1},
        combinator::map_res,
        sequence::tuple,
        IResult, Parser,
    };

    pub fn version_core(input: &str) -> IResult<&str, (usize, usize, usize)> {
        tuple((major, char('.'), minor, char('.'), patch))
            .map(|(maj, _, min, _, pat)| (maj, min, pat))
            .parse(input)
    }

    pub fn major(input: &str) -> IResult<&str, usize> {
        decimal(input)
    }

    pub fn minor(input: &str) -> IResult<&str, usize> {
        decimal(input)
    }

    pub fn patch(input: &str) -> IResult<&str, usize> {
        decimal(input)
    }

    /// Parses a decimal number with no zeroes at the start (except just '0')
    pub fn decimal(input: &str) -> IResult<&str, usize> {
        map_res(digit1, |out: &str| {
            if out.starts_with('0') {
                if out.len() == 1 {
                    Ok(0)
                } else {
                    Err(nom::Err::Failure(nom::error::Error::new(
                        out,
                        nom::error::ErrorKind::Satisfy,
                    )))
                }
            } else {
                out.parse().map_err(|_| {
                    nom::Err::Failure(nom::error::Error::new(out, nom::error::ErrorKind::Digit))
                })
            }
        })
        .parse(input)
    }
}
