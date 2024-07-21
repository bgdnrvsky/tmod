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
        tuple((
            decimal(false),
            char('.'),
            decimal(false),
            char('.'),
            decimal(false),
        ))
        .map(|(maj, _, min, _, pat)| (maj, min, pat))
        .parse(input)
    }

    /// Parses a decimal, it is possible to choose whether you want to accept zeros at the start
    pub fn decimal(accept_zeros: bool) -> impl Fn(&str) -> IResult<&str, usize> {
        move |input| {
            map_res(digit1, |out: &str| {
                if out.starts_with('0') && out.len() == 1 {
                    Ok(0)
                } else if out.starts_with('0') && !accept_zeros {
                    Err(nom::Err::Failure(nom::error::Error::new(
                        out,
                        nom::error::ErrorKind::Satisfy,
                    )))
                } else {
                    out.parse().map_err(|_| {
                        nom::Err::Failure(nom::error::Error::new(out, nom::error::ErrorKind::Digit))
                    })
                }
            })
            .parse(input)
        }
    }
}
