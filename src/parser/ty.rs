use nom::{branch::alt, combinator::map, error::context, sequence::preceded};

use crate::ast::ResolvedType;

use super::{token::*, util::located, ParseResult, Span};

fn i32_type(input: Span) -> ParseResult<ResolvedType> {
    located(map(i32, |_| ResolvedType::I32))(input)
}

fn usize_type(input: Span) -> ParseResult<ResolvedType> {
    located(map(usize, |_| ResolvedType::USize))(input)
}

fn u8_type(input: Span) -> ParseResult<ResolvedType> {
    located(map(u8, |_| ResolvedType::U8))(input)
}

fn ptr_type(input: Span) -> ParseResult<ResolvedType> {
    located(map(preceded(asterisk, parse_type), |ty| {
        ResolvedType::Ptr(Box::new(ty.value))
    }))(input)
}

pub(super) fn parse_type(input: Span) -> ParseResult<ResolvedType> {
    context("type", alt((ptr_type, i32_type, u8_type, usize_type)))(input)
}
