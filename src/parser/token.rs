use super::*;

use nom::{
    bytes::complete::{tag, take_till1},
    character::complete::{char, digit1},
    combinator::not,
};

#[inline(always)]
pub(super) fn lparen(input: Span) -> NotLocatedParseResult<char> {
    char('(')(input)
}

#[inline(always)]
pub(super) fn rparen(input: Span) -> NotLocatedParseResult<char> {
    char(')')(input)
}

#[inline(always)]
pub(super) fn lbracket(input: Span) -> NotLocatedParseResult<char> {
    char('{')(input)
}

#[inline(always)]
pub(super) fn rbracket(input: Span) -> NotLocatedParseResult<char> {
    char('}')(input)
}

#[inline(always)]
pub(super) fn comma(input: Span) -> NotLocatedParseResult<char> {
    char(',')(input)
}

#[inline(always)]
pub(super) fn colon(input: Span) -> NotLocatedParseResult<char> {
    char(':')(input)
}

#[inline(always)]
pub(super) fn equals(input: Span) -> NotLocatedParseResult<char> {
    char('=')(input)
}

#[inline(always)]
pub(super) fn plus(input: Span) -> NotLocatedParseResult<char> {
    char('+')(input)
}

#[inline(always)]
pub(super) fn minus(input: Span) -> NotLocatedParseResult<char> {
    char('-')(input)
}

#[inline(always)]
pub(super) fn asterisk(input: Span) -> NotLocatedParseResult<char> {
    char('*')(input)
}

#[inline(always)]
pub(super) fn slash(input: Span) -> NotLocatedParseResult<char> {
    char('/')(input)
}

#[inline(always)]
pub(super) fn fn_token(input: Span) -> NotLocatedParseResult<Span> {
    tag("fn")(input)
}

#[inline(always)]
pub(super) fn let_token(input: Span) -> NotLocatedParseResult<Span> {
    tag("let")(input)
}

#[inline(always)]
pub(super) fn i32(input: Span) -> NotLocatedParseResult<Span> {
    tag("i32")(input)
}

#[inline(always)]
pub(super) fn usize(input: Span) -> NotLocatedParseResult<Span> {
    tag("usize")(input)
}

#[inline(always)]
pub(super) fn u8(input: Span) -> NotLocatedParseResult<Span> {
    tag("u8")(input)
}

#[inline(always)]
pub(super) fn return_token(input: Span) -> NotLocatedParseResult<Span> {
    tag("return")(input)
}

#[inline(always)]
pub(super) fn parse_identifier(input: Span) -> NotLocatedParseResult<String> {
    let (s, _) = not(digit1)(input)?;
    map(
        take_till1(|x: char| !x.is_alphabetic() && !x.is_digit(10) && !['-', '_'].contains(&x)),
        |s: Span| s.to_string(),
    )(s)
}
