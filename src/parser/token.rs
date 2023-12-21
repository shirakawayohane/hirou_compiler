use super::*;

use nom::{
    bytes::complete::{tag, take_till1},
    character::complete::{char, digit1},
    combinator::not,
    sequence::preceded,
};

// トークン間の空白をスキップし、本筋に集中するためのコンビネーター
fn token<'a, F, O>(f: F) -> impl FnMut(Span<'a>) -> NotLocatedParseResult<()>
where
    F: FnMut(Span<'a>) -> NotLocatedParseResult<O>,
{
    map(preceded(skip0, f), |_| ())
}

macro_rules! token_char {
    ($name:ident, $arg:expr) => {
        #[inline(always)]
        pub(super) fn $name(input: Span) -> NotLocatedParseResult<()> {
            token(char($arg))(input)
        }
    };
}

macro_rules! token_tag {
    ($name:ident, $arg:expr) => {
        #[inline(always)]
        pub(super) fn $name(input: Span) -> NotLocatedParseResult<()> {
            token(tag($arg))(input)
        }
    };
}

#[test]
fn test_token_char() {
    assert!(lparen("(".into()).is_ok());
}

token_char!(lparen, '(');
token_char!(rparen, ')');
token_char!(lbracket, '{');
token_char!(rbracket, '}');
token_char!(lsqrbracket, '[');
token_char!(rsqrbracket, ']');
token_char!(langlebracket, '<');
token_char!(ranglebracket, '>');
token_char!(comma, ',');
token_char!(colon, ':');
token_char!(equals, '=');
token_char!(plus, '+');
token_char!(minus, '-');
token_char!(asterisk, '*');
token_char!(slash, '/');
token_tag!(fn_token, "fn");
token_tag!(let_token, "let");
token_tag!(i32, "i32");
token_tag!(usize, "usize");
token_tag!(u8, "u8");
token_tag!(return_token, "return");
token_tag!(doublequote, "\"");

#[inline(always)]
pub(super) fn parse_identifier(input: Span) -> NotLocatedParseResult<String> {
    let (s, _) = not(digit1)(input)?;
    map(
        take_till1(|x: char| !x.is_alphabetic() && !x.is_digit(10) && !['-', '_'].contains(&x)),
        |s: Span| s.to_string(),
    )(s)
}

#[test]
fn parse_identifier_test() {
    assert!(parse_identifier("print-i32".into()).is_ok());
    assert!(parse_identifier("buf[i]".into()).is_ok());
    assert!(parse_identifier("}".into()).is_err());
    assert!(parse_identifier("{".into()).is_err());
    assert!(parse_identifier("(".into()).is_err());
    assert!(parse_identifier(")".into()).is_err());
}
