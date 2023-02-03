use nom::{branch::alt, combinator::map, error::context, sequence::preceded};

use crate::ast::Type;

use super::{token::*, util::located, ParseResult, Span};

fn i32_type(input: Span) -> ParseResult<Type> {
    located(map(i32, |_| Type::I32))(input)
}

fn usize_type(input: Span) -> ParseResult<Type> {
    located(map(usize, |_| Type::USize))(input)
}

fn u8_type(input: Span) -> ParseResult<Type> {
    located(map(u8, |_| Type::U8))(input)
}

fn ptr_type(input: Span) -> ParseResult<Type> {
    located(map(preceded(asterisk, parse_type), |ty| {
        Type::Ptr(Box::new(ty.value))
    }))(input)
}

pub(super) fn parse_type(input: Span) -> ParseResult<Type> {
    context("type", alt((ptr_type, i32_type, u8_type, usize_type)))(input)
}
