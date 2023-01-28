use nom::{
    branch::{alt, permutation},
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    error::context,
    sequence::preceded,
};

use crate::ast::{Statement, Type};

use super::{expression::parse_expression, token::*, util::*, Located, ParseResult, Span};

fn i32_type(input: Span) -> ParseResult<Type> {
    map(i32, |_| Type::I32)(input)
}

fn u8_type(input: Span) -> ParseResult<Type> {
    map(u8, |_| Type::I32)(input)
}

fn ptr_type(input: Span) -> ParseResult<Type> {
    map(preceded(anpersand, parse_type), |ty| {
        Type::Ptr(Box::new(ty))
    })(input)
}

pub(super) fn parse_type(input: Span) -> ParseResult<Type> {
    context("type", alt((i32_type, u8_type, ptr_type)))(input)
}
