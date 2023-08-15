use nom::{
    branch::alt,
    combinator::{map, opt},
    error::context,
    multi::separated_list1,
    sequence::{delimited, pair},
};

use crate::ast::*;

use super::{token::*, util::{located, skip0}, ParseResult, Span};

fn parse_array(input: Span) -> ParseResult<UnresolvedType> {
    located(map(delimited(lsqrbracket, parse_type, rsqrbracket), |ty| {
        UnresolvedType::Ptr(Box::new(ty.value))
    }))(input)
}

fn parse_typeref(input: Span) -> ParseResult<UnresolvedType> {
    located(map(
        pair(
            parse_identifier,
            opt(delimited(
                lsqrbracket,
                separated_list1(comma, parse_typeref),
                rsqrbracket,
            )),
        ),
        |(ident, generics_args)| UnresolvedType::TypeRef(TypeRef {
            prefix: None, // TODO: impl namespace ref
            name: ident,
            generic_args: generics_args
                .map(|args| args.into_iter().map(|arg| arg.value).collect::<Vec<_>>()),
        }),
    ))(input)
}

pub(super) fn parse_type(input: Span) -> ParseResult<UnresolvedType> {
    context("type", alt((parse_array, parse_typeref)))(input)
}
