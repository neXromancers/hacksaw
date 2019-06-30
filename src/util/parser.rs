extern crate nom;
use self::nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::{complete, map},
    multi::many0,
    sequence::preceded,
    IResult,
};

#[derive(Debug)]
pub enum FormatToken {
    WindowId,
    Geometry,
    Width,
    Height,
    X,
    Y,
    Literal(String),
}

fn parse_format(input: &str) -> IResult<&str, FormatToken> {
    preceded(
        tag("%"),
        alt((
            map(tag("i"), |_| FormatToken::WindowId),
            map(tag("g"), |_| FormatToken::Geometry),
            map(tag("w"), |_| FormatToken::Width),
            map(tag("h"), |_| FormatToken::Height),
            map(tag("x"), |_| FormatToken::X),
            map(tag("y"), |_| FormatToken::Y),
            map(tag("%"), |_| FormatToken::Literal("%".to_owned())),
        )),
    )(input)
}

fn parse_reg(input: &str) -> IResult<&str, FormatToken> {
    map(is_not("%"), |s: &str| FormatToken::Literal(s.to_owned()))(input)
}

fn parse_anything(input: &str) -> IResult<&str, FormatToken> {
    alt((parse_format, parse_reg))(input)
}

pub fn parse_all(input: &str) -> IResult<&str, Vec<FormatToken>> {
    complete(many0(parse_anything))(input)
}
