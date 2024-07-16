mod version;

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

    pub fn decimal(input: &str) -> IResult<&str, usize> {
        map_res(digit1, str::parse::<usize>).parse(input)
    }
}
