use nom::{
    branch::alt,
    combinator::{map, opt},
    error::context,
    multi::separated_list1,
    sequence::{delimited, pair, preceded},
};

use crate::ast::*;

use super::{token::*, util::located, ParseResult, Span};

fn parse_ptr(input: Span) -> ParseResult<UnresolvedType> {
    located(map(preceded(asterisk, parse_type), |ty| {
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
        |(ident, generics_args)| {
            dbg!(&ident);
            UnresolvedType::TypeRef(TypeRef {
                    name: ident,
                    generic_args: generics_args
                        .map(|args| args.into_iter().map(|arg| arg.value).collect::<Vec<_>>()),
                })
        },
    ))(input)
}

pub(super) fn parse_type(input: Span) -> ParseResult<UnresolvedType> {
    context("type", alt((parse_ptr, parse_typeref)))(input)
}

#[test]
fn test_parse_type() {
    let result = parse_type(Span::new("*i32,"));
    assert!(result.is_ok());
    let (rest, ty) = result.unwrap();
    assert!(match ty.value {
        UnresolvedType::Ptr(ptr) => {
            match *ptr {
                UnresolvedType::TypeRef(TypeRef { name, generic_args }) => name == "i32" && generic_args.is_none(),
                _ => false,
            }
        },
        _ => false,
    });
    assert_eq!(rest.to_string().as_str(), ",");


    let result = parse_type(Span::new("u8,"));
    assert!(result.is_ok());
    let (rest, ty) = result.unwrap();
    assert!(match ty.value {
        UnresolvedType::TypeRef(TypeRef { name, generic_args }) => name == "u8" && generic_args.is_none(),
        _ => false,
    });
    assert_eq!(rest.to_string().as_str(), ",");
}
