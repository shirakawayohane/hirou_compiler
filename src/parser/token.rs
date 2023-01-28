use super::*;

use nom::{
    bytes::complete::{tag, take_till1},
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
pub(super) fn semicolon(input: Span) -> ParseResult<char> {
    char(';')(input)
}

#[inline(always)]
pub(super) fn colon(input: Span) -> ParseResult<char> {
    char(':')(input)
}

#[inline(always)]
pub(super) fn equals(input: Span) -> ParseResult<char> {
    char('=')(input)
}

#[inline(always)]
pub(super) fn anpersand(input: Span) -> ParseResult<char> {
    char('&')(input)
}

#[inline(always)]
pub(super) fn fn_token(input: Span) -> ParseResult<Span> {
    tag("fn")(input)
}

#[inline(always)]
pub(super) fn let_token(input: Span) -> ParseResult<Span> {
    tag("let")(input)
}

#[inline(always)]
pub(super) fn i32(input: Span) -> ParseResult<Span> {
    tag("i32")(input)
}

#[inline(always)]
pub(super) fn u64(input: Span) -> ParseResult<Span> {
    tag("u64")(input)
}

#[inline(always)]
pub(super) fn u8(input: Span) -> ParseResult<Span> {
    tag("u8")(input)
}

#[inline(always)]
pub(super) fn parse_identifier(input: Span) -> ParseResult<String> {
    let (s, _) = not(digit1)(input)?;
    map(
        take_till1(|x: char| !x.is_alphabetic() && !x.is_digit(10) && !['-', '_'].contains(&x)),
        |s: Span| s.to_string(),
    )(s)
}
