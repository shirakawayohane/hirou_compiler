use itertools::Itertools;
use nom::{
    branch::alt,
    combinator::opt,
    error::context,
    multi::separated_list1,
    sequence::{pair, preceded},
};

use crate::ast::*;

use super::*;
use super::{token::*, util::located};

pub(super) fn parse_generic_argument_decls(
    input: Span<'_>,
) -> NotLocatedParseResult<Vec<Located<GenericArgument>>> {
    fn parse_generic_argument(input: Span) -> ParseResult<GenericArgument> {
        located(context(
            "generic_argument",
            map(
                pair(
                    parse_identifier,
                    opt(preceded(colon, separated_list1(plus, parse_identifier))),
                ),
                |(name, restrictions)| GenericArgument {
                    name,
                    restrictions: restrictions
                        .unwrap_or(vec![])
                        .into_iter()
                        .map(|x| Restriction::Interface(x))
                        .collect_vec(),
                },
            ),
        ))(input)
    }
    let mut args = Vec::new();
    let (mut rest, _) = langlebracket(input)?;
    loop {
        (rest, _) = skip0(rest)?;
        if rest.starts_with('>') {
            break;
        }
        let arg;
        (rest, arg) = parse_generic_argument(rest)?;
        args.push(arg);
    }
    let (rest, _) = ranglebracket(rest)?;
    Ok((rest, args))
}

#[test]
fn test_parse_generic_argument_decls() {
    let result = parse_generic_argument_decls(Span::new("<T: a + b> { size: i32, data: T }"));
    assert!(result.is_ok());
    let (rest, args) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), " { size: i32, data: T }");
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].value.name, "T");
    assert_eq!(args[0].value.restrictions.len(), 2);
}

pub(super) fn parse_generic_arguments(
    input: Span,
) -> NotLocatedParseResult<Vec<Located<UnresolvedType>>> {
    let mut args = Vec::new();
    let (mut rest, _) = langlebracket(input)?;
    loop {
        (rest, _) = skip0(rest)?;
        if rest.starts_with('>') {
            break;
        }
        let arg;
        (rest, arg) = parse_type(rest)?;
        args.push(arg);
    }
    let (rest, _) = ranglebracket(rest)?;
    Ok((rest, args))
}

#[test]
fn test_parse_generic_arguments() {
    let result = parse_generic_arguments(Span::new("<A, B> a b)"));
    assert!(result.is_ok());
    let (rest, args) = result.unwrap();
    assert_eq!(rest.to_string().as_str(), " a b)");
}

fn parse_infer(input: Span) -> ParseResult<UnresolvedType> {
    located(map(underscore, |_| UnresolvedType::Infer))(input)
}

fn parse_ptr(input: Span) -> ParseResult<UnresolvedType> {
    located(map(preceded(asterisk, parse_type), |ty| {
        UnresolvedType::Ptr(Box::new(ty))
    }))(input)
}

fn parse_typeref(input: Span) -> ParseResult<UnresolvedType> {
    located(map(
        pair(parse_identifier, opt(parse_generic_arguments)),
        |(ident, generics_args)| {
            UnresolvedType::TypeRef(TypeRef {
                name: ident,
                generic_args: generics_args.map(|args| args.into_iter().collect::<Vec<_>>()),
            })
        },
    ))(input)
}

pub(super) fn parse_type(input: Span) -> ParseResult<UnresolvedType> {
    context("type", alt((parse_infer, parse_ptr, parse_typeref)))(input)
}

#[test]
fn test_parse_type() {
    let result = parse_type(Span::new("*i32,"));
    assert!(result.is_ok());
    let (rest, ty) = result.unwrap();
    assert!(match ty.value {
        UnresolvedType::Ptr(ptr) => {
            match &ptr.value {
                UnresolvedType::TypeRef(TypeRef { name, generic_args }) => {
                    name == "i32" && generic_args.is_none()
                }
                _ => false,
            }
        }
        _ => false,
    });
    assert_eq!(rest.to_string().as_str(), ",");

    let result = parse_type(Span::new("u8,"));
    assert!(result.is_ok());
    let (rest, ty) = result.unwrap();
    assert!(match ty.value {
        UnresolvedType::TypeRef(TypeRef { name, generic_args }) =>
            name == "u8" && generic_args.is_none(),
        _ => false,
    });
    assert_eq!(rest.to_string().as_str(), ",");

    let result = parse_type(Span::new("Vec<i32>,"));
    assert!(result.is_ok());
    let (rest, ty) = result.unwrap();
    assert!(match ty.value {
        UnresolvedType::TypeRef(TypeRef { name, generic_args }) => {
            name == "Vec" && generic_args.is_some()
        }
        _ => false,
    });
    assert_eq!(rest.to_string().as_str(), ",");
}
