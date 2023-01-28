use nom::{branch::alt, combinator::map, error::context, sequence::preceded};

use crate::ast::Type;

use super::{token::*, ParseResult, Span};

fn i32_type(input: Span) -> ParseResult<Type> {
    map(i32, |_| Type::I32)(input)
}

fn u64_type(input: Span) -> ParseResult<Type> {
    map(u64, |_| Type::U64)(input)
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
    context("type", alt((ptr_type, i32_type, u8_type, u64_type)))(input)
}
