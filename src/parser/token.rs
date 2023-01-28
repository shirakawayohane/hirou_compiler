use super::*;

use nom::{
    bytes::complete::take_till1,
    character::complete::{char, digit1},
    combinator::not,
};

#[inline(always)]
pub(super) fn lparen(input: Span) -> ParseResult<char> {
    char('(')(input)
}

#[inline(always)]
pub(super) fn rparen(input: Span) -> ParseResult<char> {
    char(')')(input)
}

#[inline(always)]
pub(super) fn lbracket(input: Span) -> ParseResult<char> {
    char('{')(input)
}

#[inline(always)]
pub(super) fn rbracket(input: Span) -> ParseResult<char> {
    char('}')(input)
}

#[inline(always)]
pub(super) fn comma(input: Span) -> ParseResult<char> {
    char(',')(input)
}

#[inline(always)]
pub(super) fn semi(input: Span) -> ParseResult<char> {
    char(';')(input)
}

#[inline(always)]
pub(super) fn equals(input: Span) -> ParseResult<char> {
    char('=')(input)
}

#[inline(always)]
pub(super) fn parse_identifier(input: Span) -> ParseResult<String> {
    let (s, _) = not(digit1)(input)?;
    map(
        take_till1(|x: char| !x.is_alphabetic() && !x.is_digit(10) && !['-', '_'].contains(&x)),
        |s: Span| s.to_string(),
    )(s)
}
